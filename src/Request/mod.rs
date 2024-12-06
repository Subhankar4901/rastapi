#![allow(dead_code)]
//! ## Request
//!
//! This module parses the incoming HTTP requests and serialize it in a **HttpRequest** struct.
//! Later this **HttpRequest** Struct is passed as the first parameter to the specific route handler function (**RouterFunction**).
use crate::{
    error, log_info, log_response_4xx,
    utils::{error::ApiError, ContentType, FileType, Method, Protocall},
    App::AppEnv,
};
use rand::{distributions::Alphanumeric, Rng};
use std::{
    borrow::Cow,
    cmp::min,
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
/// Maximum size of textual type (text,json) content to hold inside a variable in memory
/// After this limit such body contents will be streamed inside a disk.
const TEXTUAL_CONTENT_MEMORY_LIMIT: u64 = 0; //1MB
/// Chunk size of bytes read per iteration.
const BODY_READ_SPEED: u64 = 8192; //4KB
/// ## HttpRequest
/// A structure that holds details about incoming HTTP request.
pub struct HttpRequest {
    /// Protocol of incoming request, e.g. HTTP/1.0 , HTTP/1.1
    pub protocol: Protocall,
    /// Method of incoming request, e.g. GET,POST.
    pub method: String,
    /// IP address of our client. Can be None, If it's None then current request is dropped.
    pub client: Option<SocketAddr>,
    /// Quaried resource, e.g. url
    pub resource: String,
    /// Request parameters e.g. example.com?foo=bar
    pub params: Option<HashMap<String, String>>,
    /// Request Body in case of JSON/TEXT based payload. Can be None if payload is not JSON or Text.
    pub body: Option<String>,
    /// Request Body location in disk, in case of file based payload. Can be None if payload is JSON or Text type.
    pub body_location: Option<PathBuf>,
    /// Content Type of the payload.
    pub content_type: Option<ContentType>,
    /// Content Length of payload.
    pub content_len: Option<u64>,
    /// Headers of payload.
    pub headers: HashMap<String, String>,
}
impl HttpRequest {
    /// Create a new **HttpRequest** object.
    pub fn new() -> HttpRequest {
        let request_map = HttpRequest {
            protocol: Protocall::HTTP1_0,
            method: String::from(""),
            client: None,
            resource: String::from(""),
            params: None,
            body: None,
            body_location: None,
            content_type: None,
            content_len: None,
            headers: HashMap::new(),
        };
        return request_map;
    }
    /// debug function.
    fn display_req_msg(req: &Vec<u8>) {
        for byte in req {
            match byte {
                b'\r' => print!("\\r"),
                b'\n' => print!("\\n"),
                _ if byte.is_ascii() => print!("{}", *byte as char),
                _ => print!("."),
            }
        }
        println!();
    }
    /// Parse metadata (method,resource,protocol,headers) of the current request.
    fn parse_metadata<'a>(
        buffer_rdr: &mut BufReader<&TcpStream>,
        request_obj: &mut HttpRequest,
        app_env: Arc<AppEnv>,
    ) -> Result<(), ApiError<'a>> {
        let mut metadata_str = String::new();
        // Last time a byte was recieved. used in Read Time Out scenarios.
        let mut last_recieved = Instant::now();
        // Read all the metadata in a loop while reading a single line in each iteration.
        // Loop stops when encountered and empty line (/r/n), indicating end of metadata.
        loop {
            let _ = match buffer_rdr.read_line(&mut metadata_str) {
                Ok(0) => {
                    log_info!("Client Disconnected");
                    return Err(ApiError::ClientDisconnected(None));
                }
                Ok(n) => {
                    last_recieved = Instant::now();
                    if n < 3 {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if last_recieved.elapsed() > app_env.read_time_out {
                        log_info!("Request timed out.");
                        return Err(ApiError::RequestTimedout(None));
                    }
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    log_info!("Read time out hit.\n{}", e);
                    return Err(ApiError::RequestTimedout(None));
                }
                Err(e) => {
                    error!("{}", e);
                    return Err(ApiError::RequestReadError(Some(Cow::Borrowed(
                        "Error Occured while reading request meta data.",
                    ))));
                }
            };
        }
        // Parse all the recieved data and populate the HttpRequest object passed in  as a parameter as refernce.
        // Request structure :
        // METHOD Resource Protocol/r/n
        // Header-Key : Header-Value/r/n
        //.....
        //.....
        // /r/n
        let mut first_line_flag = false;
        for line in metadata_str.lines() {
            if line.trim().is_empty() {
                break;
            }
            // First line i.e. the request line parsing and validating.
            if !first_line_flag {
                match line.trim().split(' ').collect::<Vec<_>>().as_slice() {
                    [method, resorce, protocol] => {
                        request_obj.method = String::from(*method);

                        if let Some((url_part, query_part)) = resorce.split_once('?') {
                            request_obj.resource = String::from(url_part);
                            let mut param_map = HashMap::new();
                            for query in query_part.split('&') {
                                if let Some((key, val)) = query.split_once('=') {
                                    param_map.insert(key.to_string(), val.to_string());
                                }
                            }
                            request_obj.params = Some(param_map);
                        } else {
                            request_obj.resource = String::from(*resorce);
                        }
                        if let Some(proto) = Protocall::from_str(&*protocol.trim()) {
                            request_obj.protocol = proto;
                        } else {
                            return Err(ApiError::RequestNotHttp(Some(Cow::Borrowed(
                                "Only HTTP/1.x requests are allowed.",
                            ))));
                        }
                    }
                    _ => {
                        return Err(ApiError::RequestNotHttp(Some(Cow::Borrowed(
                            "Only HTTP/1.x requests are allowed.",
                        ))));
                    }
                }
                let route = match unsafe { app_env.routes.as_ref() }
                    .get_route(&request_obj.resource, None)
                {
                    Some(r) => r,
                    None => {
                        let error_msg = format!("Resource {} not found", request_obj.resource);
                        log_response_4xx!(
                            request_obj.method,
                            request_obj.resource,
                            404,
                            "Resource not found."
                        );
                        return Err(ApiError::ResourceNotFound(Some(Cow::Owned(error_msg))));
                    }
                };
                let method = match Method::from_string(request_obj.method.as_str()) {
                    Some(m) => m,
                    None => {
                        let error_msg = format!("Method {} not supported.", request_obj.method);
                        log_response_4xx!(
                            request_obj.method,
                            request_obj.resource,
                            405,
                            "Method Not Supported."
                        );
                        return Err(ApiError::MethodNotSupported(Some(Cow::Owned(error_msg))));
                    }
                };
                if !route.methods.contains(&method) {
                    let error_msg = format!("Method {} not allowed.", request_obj.method);
                    log_response_4xx!(
                        request_obj.method,
                        request_obj.resource,
                        405,
                        "Method Not Allowed."
                    );
                    return Err(ApiError::MethodNotAllowed(Some(Cow::Owned(error_msg))));
                }
                first_line_flag = true;
            }
            // Header parsing and validating.
            if let Some((key, val)) = line.trim().split_once(": ") {
                match key {
                    "Content-Length" | "content-length" if request_obj.method != String::from("GET") => {
                        let size = match val.parse::<u64>() {
                            Ok(n) => n,
                            Err(_e) => {
                                return Err(ApiError::InvalidContentLength(None));
                            }
                        };
                        if size > app_env.maximum_pay_load_in_bytes as u64 {
                            let _max_mb = app_env.maximum_pay_load_in_bytes / (1024 * 1024);
                            log_response_4xx!(
                                request_obj.method,
                                request_obj.resource,
                                413,
                                "Payload too large."
                            );
                            return Err(ApiError::PayloadTooLarge(None));
                        }
                        request_obj.content_len = Some(size);
                        request_obj
                            .headers
                            .insert(String::from(key), String::from(val));
                    }
                    "Content-Type" | "content-type" if request_obj.method != String::from("GET") => {
                        let content_type = match ContentType::from_header(val) {
                            Ok(ct) => ct,
                            Err(_e) => {
                                return Err(ApiError::ContentNotSupported(None));
                            }
                        };
                        request_obj.content_type = Some(content_type);
                        request_obj
                            .headers
                            .insert(String::from(key), String::from(val));
                    }
                    _ => {
                        request_obj.headers.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
        Ok(())
    }
    // Parse File type payload.
    fn read_body_to_file<'a>(
        buffer_rdr: &mut BufReader<&TcpStream>,
        request_obj: &mut HttpRequest,
        filetype: FileType,
        incoming_dir:&str
    ) -> Result<(), ApiError<'a>> {
        let mut file_path = match env::current_dir(){
            Ok(p)=>p,
            Err(e)=>{
                error!("{}",e);
                return Err(ApiError::CannotWriteDataToDisk(Some(Cow::Borrowed("Failed to get the current directory"))));
            }
        };
        file_path.push(incoming_dir);
        if !file_path.exists(){
            match fs::create_dir(&file_path){
                Ok(_)=>(),
                Err(e)=>{
                    error!("{}",e);
                    return Err(ApiError::CannotWriteDataToDisk(Some(Cow::Borrowed("Failed to create incoming directory."))));
                }
            };
        }
        let mut file_name: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        file_name.push_str(filetype.extension());
        file_path.push(file_name);
        let dest_file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("{}", e);
                return Err(ApiError::RequestBodyNotRead(None));
            }
        };
        let mut dest_file_buff = BufWriter::new(dest_file);
        let chunk_size = BODY_READ_SPEED;
        let content_len = match request_obj.content_len {
            Some(n) => n,
            None => 0 as u64,
        };
        let mut bytes_left_to_read = content_len;
        while bytes_left_to_read > 0 {
            let temp_buf_size = min(bytes_left_to_read, chunk_size);
            let mut temp_buf = vec![0 as u8; temp_buf_size as usize];
            let _ = match buffer_rdr.read_exact(&mut temp_buf) {
                Ok(_) => {
                    match dest_file_buff.write_all(&mut temp_buf) {
                        Ok(_) => (),
                        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                            error!(
                                "You don't have permission to write to disk. file location : {:?}",
                                &file_path
                            );
                            return Err(ApiError::CannotWriteDataToDisk(None));
                        }
                        Err(e) => {
                            error!("{}", e);
                            return Err(ApiError::CannotWriteDataToDisk(None));
                        }
                    }
                    match dest_file_buff.flush() {
                        Ok(_) => (),
                        Err(e) => {
                            error!("{}", e);
                            return Err(ApiError::CannotWriteDataToDisk(None));
                        }
                    }
                    bytes_left_to_read -= temp_buf_size;
                }
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    error!("Read time out reached.\n{}", e);
                    return Err(ApiError::RequestTimedout(None));
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    error!("UnexpectedEOF : Connection may be dropped prematurely.");
                    return Err(ApiError::ClientDisconnected(None));
                }
                Err(e) => {
                    error!("{}", e);
                    return Err(ApiError::RequestBodyNotRead(None));
                }
            };
        }
        request_obj.body_location = Some(file_path);
        Ok(())
    }
    // Parse JSON/ Text based payload.
    fn parse_text<'a>(
        buffer_rdr: &mut BufReader<&TcpStream>,
        request_obj: &mut HttpRequest,
    ) -> Result<(), ApiError<'a>> {
        let mut final_buf: Vec<u8> = Vec::new();
        let chunk_size: u64 = BODY_READ_SPEED;
        let content_len = match request_obj.content_len {
            Some(n) => n,
            None => 0 as u64,
        };
        let mut bytes_left_to_read = content_len;
        while bytes_left_to_read > 0 {
            let temp_buf_len = min(chunk_size, bytes_left_to_read);
            let mut temp_buf = vec![0 as u8; temp_buf_len as usize];
            let _ = match buffer_rdr.read_exact(&mut temp_buf) {
                Ok(_) => {
                    final_buf.append(&mut temp_buf);
                    bytes_left_to_read -= temp_buf_len;
                }
                Err(e)
                    if e.kind() == io::ErrorKind::TimedOut
                        || e.kind() == io::ErrorKind::WouldBlock =>
                {
                    error!("Read timeout reached.\n {}", e);
                    return Err(ApiError::RequestTimedout(None));
                }
                Err(e) => {
                    error!("{}", e);
                    return Err(ApiError::RequestBodyNotRead(Some(Cow::Borrowed(
                        "Couldn't read request body.",
                    ))));
                }
            };
        }
        // Self::display_req_msg(&final_buf);
        let body = match String::from_utf8(final_buf) {
            Ok(s) => s,
            Err(e) => {
                error!("{}", e);
                return Err(ApiError::RequestDataNotUTF8(Some(Cow::Borrowed(
                    "Request body not UTF-8 complient.",
                ))));
            }
        };
        request_obj.body = Some(body);

        Ok(())
    }
    /// Create a HttpRequest object from a TcpStream. i.e. read the data coming from the stream and
    /// build the HttpRequest Object step by step.
    pub(crate) fn from_tcp_stream<'a>(
        stream: Arc<TcpStream>,
        app_env: Arc<AppEnv>,
    ) -> Result<HttpRequest, ApiError<'a>> {
        let _ = stream.set_read_timeout(Some(app_env.read_time_out));
        let mut request_obj: HttpRequest = HttpRequest::new();
        request_obj.client = match stream.peer_addr() {
            Ok(addr) => Some(addr),
            Err(e) => {
                error!("{}", e);
                return Err(ApiError::ClientNotFound(None));
            }
        };
        let mut buffer_rdr = BufReader::new(stream.as_ref());
        match Self::parse_metadata(&mut buffer_rdr, &mut request_obj, app_env.clone()) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };
        if request_obj.method != String::from("GET") {
            if let Some(ref content_type) = request_obj.content_type {
                if let Some(content_len) = request_obj.content_len {
                    match content_type {
                        textual_content @ (ContentType::TEXT
                        | ContentType::JSON
                        | ContentType::YAML) => {
                            if content_len < TEXTUAL_CONTENT_MEMORY_LIMIT {
                                match Self::parse_text(&mut buffer_rdr, &mut request_obj) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            } else {
                                let filetype = textual_content.to_file_type();

                                match Self::read_body_to_file(
                                    &mut buffer_rdr,
                                    &mut request_obj,
                                    filetype,
                                    &app_env.incoming_file_directory
                                ) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                        }
                        bytes_content @ _ => {
                            let filetype = bytes_content.to_file_type();

                            match Self::read_body_to_file(
                                &mut buffer_rdr,
                                &mut request_obj,
                                filetype,
                                &app_env.incoming_file_directory,
                            ) {
                                Ok(_) => (),
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            } else {
                return Err(ApiError::ContentTypeRequired(None));
            }
        }
        Ok(request_obj)
    }
}
