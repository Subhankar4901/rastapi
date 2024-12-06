#![allow(dead_code)]
//! ## Response
//!
//! This module handles creating and sending HTTP responses.
pub(crate) mod pre_built_resp;
// pub mod response;
extern crate chrono;
use crate::cache::{Cache,FileData};
use crate::error;
use crate::utils::{ContentType, HTTPCode};
use crate::utils::{FileType, Protocall};
use crate::File::FileWrapper;
use chrono::format::strftime::StrftimeItems;
use chrono::offset::Utc;
use pre_built_resp::{InternalServerError, Notfound404};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::net::TcpStream;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use std::{io::Write, path::PathBuf};
/// This is the main struct that creates a Response object. And it also handles sending responses.
pub struct HttpResponse {
    /// HTTP response code. e.g. 200,404,503 etc.
    pub(crate) code: HTTPCode,
    /// Content type of the response payload. e.g. JSON,TEXT,MP4,MP3 etc.
    pub content_type: ContentType,
    /// Payload length.
    pub content_len: usize,
    /// The response payload if Content type is JSON or TEXT  else it's None.
    pub content: Option<String>,
    /// Boolean flag to determine if response is a file response.
    pub file_response: bool,
    /// If response is a file response then it stores location of the file else it's None.
    pub file_content_location: Option<PathBuf>,
    /// If response is a file response then it stores the name of the file else it's None.
    pub file_name: Option<String>,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Flag to determine if we keep the connection alive after response or not.
    pub keep_alive: bool,
}
impl HttpResponse {
    /// Create a new HttpResponse object.
    pub(crate) fn new(
        resp_code: HTTPCode,
        content_type: ContentType,
        content_len: usize,
        content: Option<String>,
        file_response: bool,
        file_content_location: Option<PathBuf>,
        filename: Option<String>,
        keep_alive: bool,
    ) -> HttpResponse {
        let headers = HashMap::new();

        HttpResponse {
            code: resp_code,
            content_type: content_type,
            content_len: content_len,
            content: content,
            file_response: file_response,
            file_content_location: file_content_location,
            headers: headers,
            file_name: filename,
            keep_alive: keep_alive,
        }
    }
    /// Function to add headers.
    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }
    fn validate_etag(
        file_len: usize,
        last_updated: usize,
        recieved_etag: Option<String>,
    ) -> (bool, String) {
        let mut cur_etag = last_updated.to_string();
        cur_etag.push('@');
        cur_etag.push_str(&file_len.to_string());
        if recieved_etag.is_none() {
            (false, cur_etag)
        } else {
            (cur_etag == recieved_etag.unwrap(), cur_etag)
        }
    }
    fn to_string(&self, protocall: Protocall) -> String {
        let mut resp = String::new();
        resp.push_str(protocall.to_str());
        resp.push(' ');
        resp.push_str(&format!("{} {}\r\n", self.code as u16, self.code.get_msg()));
        let cur_time =
            Utc::now().format_with_items(StrftimeItems::new("%a, %d %b %Y %H:%M:%S GMT"));
        resp.push_str(&format!("Date: {}\r\n", cur_time));
        for (key, val) in self.headers.iter() {
            resp.push_str(&format!("{}: {}\r\n", key, val));
        }
        let content_type = self.content_type.get_content_type_header();
        resp.push_str(content_type);
        resp.push_str("\r\n");
        resp
    }
    /// Function to send response.
    pub(crate) fn send_response(
        &mut self,
        stream: Arc<TcpStream>,
        protocall: Protocall,
        write_time_out: Duration,
        cache: NonNull<Cache<PathBuf>>,
        etag_value_recieved: Option<String>,
        send_buffer_size:usize
    ) -> Result<u16, io::Error> {
        if let Some(content) = &self.content {
            let mut resp = self.to_string(protocall);
            resp.push_str(content);
            let resp_bytes = resp.as_bytes();
            let mut resp_len = resp_bytes.len();
            let mut last_updated = Instant::now();
            while resp_len > 0 {
                match stream.as_ref().write(&resp_bytes[resp.len() - resp_len..]) {
                    Ok(n) => {
                        resp_len -= n;
                        last_updated = Instant::now();
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        if last_updated.elapsed() > write_time_out {
                            return Err(e);
                        }
                        continue;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        } else {
            if self.file_response {
                return self.send_file_response(
                    stream,
                    protocall,
                    write_time_out,
                    cache,
                    etag_value_recieved,
                    send_buffer_size
                );
            }
        }
        Ok(self.code as u16)
    }
    /// Function to send file responses.
    fn send_file_response(
        &mut self,
        stream: Arc<TcpStream>,
        _protocall: Protocall,
        write_time_out: Duration,
        #[allow(unused_variables)]
        cache: NonNull<Cache<PathBuf>>,
        etag_value_recieved: Option<String>,
        send_buffer_size:usize
    ) -> Result<u16, io::Error> {
        if let Some(file_location) = self.file_content_location.clone() {
            let file = File::open(&file_location)?;
            let (file_len, file_last_updated) = match file.metadata() {
                Ok(m) => (
                    m.len(),
                    m.modified()?.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                ),
                Err(e) => {
                    return Err(e);
                }
            };

            let (valid, cur_etag) = Self::validate_etag(
                file_len as usize,
                file_last_updated as usize,
                etag_value_recieved,
            );

            if valid {
                // Etag valid. No need send file.
                    return Self::send_304_response(stream.clone(), _protocall, cur_etag);
            } 
            
            else {
                self.add_header("Etag", &cur_etag);

                // Construct metadata of response.
                let metadata = self.to_string(_protocall);

                // Check if file in cache or not.
                #[cfg(feature="caching")]
                if let Some(cache_res) = unsafe { cache.as_ref() }.get(file_location.clone()){
                    
                    // Check if the cache we found is valid or not.
                    if cache_res.val.len()==file_len as usize{

                        // First we send the response metadata.
                            let _=match Self::send_response_metadata(stream.clone(), &metadata, send_buffer_size,write_time_out){
                                Ok(())=>(),
                                Err(e)=>{
                                    return Err(e);
                                }
                            };
                        // Then we send the file from cache.
                            return self.send_cached_response(stream.clone(), &cache_res, send_buffer_size, write_time_out);
                    }
                
                };
                // Now if file is not in cache or cache is invalid
                // Send file from disk.
                // First we send response metadata.
                let _=match Self::send_response_metadata(stream.clone(), &metadata, send_buffer_size,write_time_out){
                    Ok(())=>(),
                    Err(e)=>{
                        return Err(e);
                    }
                };
                // Then we send the file from disk.
                #[allow(unused_variables)]
                let file_data=match Self::write_file_from_disk_to_network(stream.clone(), file, send_buffer_size, write_time_out){
                    Ok(v)=>{
                        v.into_vec()
                    }
                    Err(e)=>{
                        return Err(e);
                    }
                };
                
                // After sending we insert the file into cache.
                #[cfg(feature="caching")]
                unsafe { cache.as_ref() }.insert(file_location.clone(), &file_data);
            }
            return Ok(self.code as u16);
        }
        Err(io::ErrorKind::NotFound.into())
    }
    fn send_304_response(stream: Arc<TcpStream>,protocall: Protocall,etag:String)->Result<u16,io::Error>
    {
        let mut resp = String::new();
        resp.push_str(protocall.to_str());
        resp.push(' ');
        resp.push_str(&format!("{} {}\r\n", 304, HTTPCode::NotModified.get_msg()));
        let cur_time =
            Utc::now().format_with_items(StrftimeItems::new("%a, %d %b %Y %H:%M:%S GMT"));
        resp.push_str(&format!("Date: {}\r\n", cur_time));
        resp.push_str(&format!("Etag: {}\r\n", &etag));
        resp.push_str("\r\n");
        let _ = match stream.as_ref().write_all(resp.as_bytes()) {
            Ok(_) => {
                return Ok(304);
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
    fn send_cached_response(&self,stream: Arc<TcpStream>,data:&FileData,send_buffer_size:usize,write_time_out: Duration)->Result<u16,io::Error>{
      for chunk in data.val.chunks(send_buffer_size){
        let mut bytes_left_to_write:usize=0;
        let total_bytes_to_write=chunk.len();
        let mut last_written=Instant::now();
        while bytes_left_to_write<total_bytes_to_write {
            let _=match stream.as_ref().write(&chunk[bytes_left_to_write..]){
                Ok(0)=>{
                    return Err(io::ErrorKind::WriteZero.into());
                },
                Ok(n)=>{
                last_written=Instant::now();
                bytes_left_to_write+=n;
            },
            Err(e) if e.kind()==io::ErrorKind::WouldBlock=>{
            if last_written.elapsed()>write_time_out{
                return Err(e);
            }
            continue;
            },
            Err(e)=>{
                return Err(e);
            }

        };
      }
    }
    Ok(self.code as u16)
}
fn send_response_metadata(stream: Arc<TcpStream>,metadata:&str,send_buffer_size:usize,write_time_out: Duration)->Result<(),io::Error>{
    let metadat_bytes=metadata.as_bytes();
    for chunk in metadat_bytes.chunks(send_buffer_size){

        let mut bytes_written=0_usize;
        let mut last_written=Instant::now();
        let total_bytes_to_write=chunk.len();
        while  bytes_written<total_bytes_to_write{
            let _=match stream.as_ref().write(&chunk[bytes_written..]){
                Ok(n)=>{
                    if n==0{
                        return Err(io::ErrorKind::WriteZero.into());
                    }
                    last_written=Instant::now();
                    bytes_written+=n;
                }
                Err(e) if e.kind()==io::ErrorKind::WouldBlock=>{
                    if last_written.elapsed() > write_time_out{
                        return Err(io::ErrorKind::WouldBlock.into());
                    }
                    continue;
                }
                Err(e)=>{
                    return Err(e);
                }
            };
        }
    }
    Ok(())
}
fn write_file_from_disk_to_network(stream:Arc<TcpStream>,file:File,send_buffer_size:usize,write_time_out: Duration)->Result<Box<[u8]>,io::Error>{
    #[allow(unused_mut)]
    let mut file_data = Vec::<u8>::new();
    let mut file_wrapper = FileWrapper::new(file, Some(send_buffer_size));
    for chunk in file_wrapper.iter() {
        match chunk {
            Ok(v) => {
                #[cfg(feature="caching")]
                file_data.extend_from_slice(&v);
                let mut bytes_written=0_usize;
                let total_bytes_to_write = v.len();
                let mut last_written = Instant::now();
                while bytes_written < total_bytes_to_write {
                    match stream.as_ref().write(&v[bytes_written..]){
                        Ok(n)=>{
                            if n==0
                            {
                                return Err(io::ErrorKind::WriteZero.into());
                            }
                            last_written=Instant::now();
                            bytes_written+=n;
                        }
                        Err(e) if e.kind()==io::ErrorKind::WouldBlock=>{
                            if last_written.elapsed() > write_time_out{
                                return Err(e);
                            }
                            continue;
                        },
                        Err(e)=>{
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(file_data.into_boxed_slice())
}
}
/// Creates a new response object for JSON and TEXT type responses.
/// ## Parameters
///  - `content` : JSON or TEXT payload.
///  - `http_code` : HTTP response Code.
///  - `content_type` : Type of payload. It can either be *ContentType::TEXT* or *ContentType::JSON*.
///  - `keep_alive` : Keep alive flag it can be *true* for persistent connection or *false* for non-persistent connection.
/// ## Return
///  - `Result<HttpResponse,io::Error>`
///
/// It returns an error if the provided HTTP code is not implimented.
///
/// ## Example
/// ```no_run
/// use rastapi::{Response::create_response,utils::ContentType};
/// let resp=create_response("Hello World",200,ContentType::TEXT,false).unwrap();
/// ```
pub fn create_response(
    content: &str,
    http_code: u16,
    content_type: ContentType,
    keep_alive: bool,
) -> Result<HttpResponse, io::Error> {
    let resp_code = match HTTPCode::from_u16(http_code) {
        Some(c) => c,
        None => {
            error!("HTTP Code of {} is not implimented.", http_code);
            return Err(io::ErrorKind::InvalidData.into());
        }
    };
    let content_len = content.len();
    let mut resp = HttpResponse::new(
        resp_code,
        content_type,
        content_len,
        Some(content.to_string()),
        false,
        None,
        None,
        keep_alive,
    );
    resp.add_header("Content-Length", content_len.to_string().as_str());
    return Ok(resp);
}
/// Create a new response object for file type responses.
/// ## Parameters
///  - `file_location` : Absolute location of a file.
///  - `file_name` : Name of the file.When Client recieves the file this name will show.It can be left None,in that case name will be infered from path.
///  - `file_type` : Type of the file. Don't confuse it with *std::fs::FileType*. It's a enum to indicate file format. e.g. FileType::MP4.
///  - `http_code` : HTTP response Code.
///  - `keep_alive` : Keep alive flag it can be *true* for persistent connection or *false* for non-persistent connection.
/// ## Return
///  - `Result<HttpResponse,io::Error>`
///
/// It returns an error if the HTTP code is not implimented.
/// It returns 404 Not Found if the file doesn't exists or Some error occurs while fetching file data.
/// ## Example
/// ```no_run
/// use rastapi::{Response::send_file,utils::FileType};
/// let resp=send_file("absolute/path/to/file",Some("filename.mp4".to_string()),FileType::MP4,200,true).unwrap(); // resp is a HttpResponse type.
/// ```
pub fn send_file(
    file_location: &str,
    mut file_name: Option<String>,
    file_type: FileType,
    http_code: u16,
    keep_alive: bool,
) -> Result<HttpResponse, io::Error> {
    let resp_code = match HTTPCode::from_u16(http_code) {
        Some(c) => c,
        None => {
            error!("HTTP Code of {} is not implimented.", http_code);
            return Err(io::ErrorKind::InvalidData.into());
        }
    };
    let file_location_path_buf = PathBuf::from(file_location);
    let file = match File::open(&file_location) {
        Ok(f) => f,
        Err(_e) => {
            return Ok(Notfound404(Cow::Borrowed(
                "File doesn't exist or has been deleted.",
            )));
        }
    };
    let file_metadata = match file.metadata() {
        Ok(m) => m,
        Err(_e) => {
            return Ok(InternalServerError(""));
        }
    };
    let file_len = file_metadata.len();
    if file_name.is_none() {
        file_name = file_location_path_buf
            .file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned());
    }
    let mut resp = HttpResponse::new(
        resp_code,
        file_type.to_content_type(),
        file_len as usize,
        None,
        true,
        Some(file_location_path_buf),
        file_name,
        keep_alive,
    );
    resp.add_header("Content-Length", file_len.to_string().as_str());
    if let Some(_f_) = &resp.file_name {
        let val = format!(r#"attachment; filename="{}""#, _f_);
        resp.add_header("Content-Disposition", val.as_str());
    }
    Ok(resp)
}
