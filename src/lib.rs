#![allow(non_snake_case)]
//! # RastAPI
//!
//! RastAPI is a easy-to-use Rust library for creating low-boilerplate, efficient RESTfull APIs, inspired by FastAPI (Python).
//! You define route handler (functions) and map them with their corresponding url and you have a functioning route.
//! RastAPI follows a **Non Blocking IO** structure for efficient and conccurent handling of requests.
//! We achieve Non blocking IO by following a **Threadpool** architecture. We can set the number of threads in the pool before
//! starting the application.
//! RastAPI also has a very efficient LFU-LRU based file caching system.It's a multi threaded cache system, so we used efficient lock
//! stripping techniques to reduce lock contentions.
//!
//! ## Features
//!  - Minimal boilerplate for defining routes and handlers.
//!  - High performance leveraging Rust's coccurency and safety.
//!  - Very efficient file uploads and downloads.
//!
//! ## Example
//! ### Text or Json Response
//!
//! ```no_run
//!     use rastapi::RastAPI;
//!     use rastapi::Request::HttpRequest;
//!     use rastapi::Response::{HttpResponse,create_response};
//!     use rastapi::utils::ContentType;
//!     use std::collections::HashMap;
//!     
//!     // Define a Function/Route handler.
//!     // With this exact fucnction signature.
//!     // i.e. fn function_name(request_obj:&HttpRequest,path_params:HashMap<String,String>)->HttpResponse
//!     // request_obj is parameter where all the details of the incoming request is stored.
//!     // path_params is a parameter where url path params are stored. Not confuse it with url query parameters.
//!     // query params are stored in request_obj.params.
//!
//!     fn json(request_obj:&HttpRequest,path_params:HashMap<String,String>)->HttpResponse{
//!
//!         let json_content=r#"{
//!         "name" : "Rony",
//!         "batch" : 2024,
//!         "sem" : 3,
//!         "subjects" : ["OS","Networking","Algorithms"],
//!         "grade" : {
//!         "OS" : "C",
//!         "Cryptography" : "C",
//!         "Algorithms" : "C"
//!                    }           
//!         }"#;
//!
//!         let resp=create_response(&json_content, 200,ContentType::JSON,true).unwrap(); //create_response(string_content,HTTP_CODE,
//!         return resp;                                                                  // ContentType,keep_alive_flag)
//!     }
//!     fn main(){
//!         let mut app=RastAPI::new();
//!         // Set number of workers. It indicates total number of simultaneous requests it can serve.
//!         app.set_total_workers(5);
//!         // Map the function/route handler with corresponding url route.
//!         app.register_route("/json",vec!["GET"],json);
//!         app.run("0.0.0.0",5000);
//!     }
//!```
//! ### File response
//!
//! One of the most powerfull features of rastapi is It's very efficient file uploads downloads. If a file is Not found on system
//! then the server automatically sends 404 Not Found response.
//!
//! ```no_run
//!     use rastapi::RastAPI;
//!     use rastapi::Request::HttpRequest;
//!     use rastapi::Response::{HttpResponse,send_file};
//!     use rastapi::utils::FileType;
//!     use std::collections::HashMap;
//!
//!     fn file(req:&HttpRequest,path_params:HashMap<String,String>)->HttpResponse{
//!     let mut resp=send_file("file_path/file_name.ext",Some("file_name.ext".to_string()),FileType::MP4,200,true).unwrap(); // send_file0(file_path,file_name
//!     resp.add_header("Header-Name","Header-Value");                                              //  file_type,HTTP_CODE,keep_alive_flag)
//!     return resp;
//!     }
//!     fn main(){
//!         let mut app=RastAPI::new();
//!         app.set_total_workers(5);
//!         app.register_route("/file",vec!["GET"],file);
//!         app.run("127.0.0.1",5000);
//!      }
//! ```
//! ### Load local enviornment variables and access them from std::env module.
//! ```no_run
//!     use rastapi::RastAPI;
//!     use rastapi::Request::HttpRequest;
//!     use rastapi::Response::{HttpResponse,send_file};
//!     use rastapi::utils::FileType;
//!     use rastapi::utils::load_env;
//!     use std::collections::HashMap;
//!     use std::env;
//!     fn file(req:&HttpRequest,path_params:HashMap<String,String>)->HttpResponse{
//!     let mut resp=send_file("file_path/file_name.ext",Some("file_name.ext".to_string()),FileType::MP4,200,true).unwrap(); // send_file0(file_path,file_name
//!     resp.add_header("Header-Name","Header-Value");                                              //  file_type,HTTP_CODE,keep_alive_flag)
//!     return resp;
//!     }
//!     fn main(){
//!         load_env::load_env(); // Loads all local enviornment variables in .env file.
//!         let mut app=RastAPI::new();
//!         app.set_total_workers(5);
//!         app.register_route("/file",vec!["GET"],file);
//!         let port:u16=env::var("PORT").unwrap().parse().unwrap();
//!         app.run(env::var("HOST").unwrap().as_str(),port);
//!      }
//! ```
//! ## Cache
//! RastAPI uses a slightly tweaked version of a standard LFU-LRU (LFU for eviction and LRU when there is a tie between frequencies of two entity) cache. It only caches files for now.
//! It's a multi threaded cache So for syncronization we use locks (Mutex). We cann't use RwLock as for LRU-LFU cache as every read qyery is a write query.
//! So there is a issue of Lock contention. To mitigate lock contention we devided the cache into multiple parts (CacheStore) and lock them individually, thus reducing load on
//! single cache store.
//! We can set total cache size and total number of cache stores.
//!
//! ```no_run
//!     use rastapi::RastAPI;
//!     fn main(){
//!     let mut app=RastAPI::new();
//!     #[cfg(feature="cachung")]
//!      app.set_cache_config(500,10); // Total cache size is set to 500 MB and
//!                                    // total number of cache stores set to 10
//! }
//! ```
//! 
//! ## Persistent Connections
//!  RastAPI supports persistent connections. If you want a route to be persistent then set the `keep_alive` flag in `create_response` or `send_file` as `true`.
//!  You can tweak the keep alive behabiour like maximum number of keep alive requests i.e. total number of request response cycle on a signle connection can be set,
//!  default value is 10. You can also set the keep alive timeout, i.e. for how much time we wait for next request after sending a response on persistent connection.
//!  If we don't recieve new request for this duration then we drop the connection. Default value is 5 seconds.
//! 
//! ```no_run
//!     use rastapi::RastAPI;
//!     fn main(){
//!     let mut app=RastAPI::new();
//!     app.set_keep_alive_time_out(3);
//!     app.set_maximum_keep_alive_requests(10);
//!     app.run("127.0.0.1",5000);
//! }
//! ```
//! ## Stop the app
//!  To stop the app gracefully you need to send SIGINT (CTRL + C) or SIGTERM. SIGTSTP(CTRL + Z) is ignored.
mod App;
mod File;
pub mod Request;
pub mod Response;
mod cache;
mod macros;
pub mod utils;
use cache::Cache;
use libc::{
    sigaction as sigaction_syscall, sigaction as sigaction_struct, sigaddset, SA_RESTART, SIGINT,
    SIGTERM, SIGTSTP,getsockopt,SOL_SOCKET,SO_SNDBUF
};
use std::{
    ffi::c_void, io, net::{IpAddr, TcpListener, TcpStream, UdpSocket}, os::fd::AsRawFd, path::PathBuf, process, ptr::NonNull, sync::Arc, time::Duration
};
use utils::{threadpool::ThreadPool, Method};
use App::{client::Client, AppEnv, Route, RouteFunction, URLRouter};

// Signal handling ctrl+c & ctrl + z
static mut SIG_FLAG: bool = false;
static mut PORT: u16 = 0;
extern "C" fn handle_shutdown_signal(_: i32) {
    unsafe {
        SIG_FLAG = true;
        let addr = format!("127.0.0.1:{}", PORT);
        let _dummy_stream = match TcpStream::connect(&addr) {
            Ok(c) => c,
            Err(_) => {
                process::exit(1);
            }
        };
    }
}
extern "C" fn handle_sigtstp(_: i32) {
    log_info!("CTRL + Z is ignored. If want to terminate the server press CTRL + C");
}
fn get_send_buffer_len(sock_fd:i32)->usize{
    let mut buf_sz=1<<14;
    unsafe {
        let n_ptr=Box::into_raw(Box::new(0_usize)) as * mut c_void;
        let m_ptr=Box::into_raw(Box::new(std::mem::size_of::<usize>() as u32));
        let res=getsockopt(sock_fd, SOL_SOCKET, SO_SNDBUF, n_ptr, m_ptr);
        if res==0{
            buf_sz=*(n_ptr as * const usize);
        }
        let _=Box::from_raw(n_ptr as * mut usize);
        let _=Box::from_raw(m_ptr);
    }
    if buf_sz>(1<<16){
        buf_sz=1<<14;
    }
    buf_sz as usize
}
/// This is the main API struct. Before doing anything we need to initialize it.
/// ## Example
/// ```no_run
/// use rastapi::RastAPI;
///     use rastapi::Request::HttpRequest;
///     use rastapi::Response::{HttpResponse,send_file};
///     use rastapi::utils::FileType;
///     use rastapi::utils::load_env;
///     use std::collections::HashMap;
///
///     fn route_handler(req:&HttpRequest,path_params:HashMap<String,String>)->HttpResponse{
///     let mut resp=send_file("file_path/file_name.ext",Some("file_name.ext".to_string()),FileType::MP4,200,true).unwrap(); // send_files(file_path,file_name
///     resp.add_header("Header-Name","Header-Value");                                              //  file_type,HTTP_CODE,keep_alive_flag)
///     return resp;
///      }
///
/// fn main(){
///     let mut app=RastAPI::new();
///     app.set_total_workers(5);
///     app.set_maximum_payload_size(100);
///     app.set_read_time_out(5);
///     app.set_write_time_out(5);
///     app.set_keep_alive_time_out(5);
///     app.set_maximum_keep_alive_requests(10);
///     app.register_route("/route",vec!["GET"],route_handler);
///     app.run("localhost",5000);
/// }
///    
///
/// ```
pub struct RastAPI {
    /// Routes of our app.
    pub(crate) routes: NonNull<URLRouter>,
    /// Total number of workers. i.e. total threads in our threadpool. It's system threads NOT green threads. Default 10 workers.
    pub total_workers: usize,
    /// Maximum size of payload to accept on each request. If it exceeds for any request our API will automatically send *413 Payload too larrge*. Default 512 MB.
    pub payload_maximum_size_in_MB: usize,
    /// If no packet recieved for this duration then we stop reading from stream and drop the connection. Default 5 secs.
    pub read_time_out: Duration,
    /// If we couldn't write on the stream for this duration we stop sending request and drop the connection. Default 5 secs.
    pub write_time_out: Duration,
    /// Maximum amount of time we keep a persistent connection alive after sending a response. Default 5 secs.
    pub keep_alive_time_out: Duration,
    /// Maximum number of request-response cycle on a persistent connection. Default 10 request-response cycle.
    pub keep_alive_max_count: u8,
    /// A LFU-LRU cache for file caching. Default size 400 MB, devided among 10 Cache Stores.
    pub(crate) cache: NonNull<Cache<PathBuf>>,
    /// Name of the directory where incoming files are stored. i.e. files coming in request bodies. Default name is `input_files`.
    pub file_upload_directory_name:String
}
impl RastAPI {
    /// Initializes a RastAPI struct with default configurations.
    pub fn new() -> Self {
        let total_workers: usize = 10;
        let default_read_time_out = Duration::from_secs(5);
        let deafault_write_time_out = Duration::from_secs(5);
        let deafault_keep_alive_time_out = Duration::from_secs(5);
        Self {
            routes: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(URLRouter::new()))) },
            total_workers,
            payload_maximum_size_in_MB: 512 as usize,
            read_time_out: default_read_time_out,
            write_time_out: deafault_write_time_out,
            keep_alive_time_out: deafault_keep_alive_time_out,
            keep_alive_max_count: 10,
            cache: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Cache::new(10, 40)))) },
            file_upload_directory_name:String::from("input_files")
        }
    }

    /// Sets the cache configuration.
    /// ## Parameters
    ///  - `total_cache_size` : Total size of the cache.
    ///  - `total_no_of_cache_stores` : Total number of cache stores. each cache store will contain part of the cache.
    ///   Maximum size of a cache store will be `toatal_cache_size`/`total_no_of_cache_stores`.
    ///
    /// Make sure `total_cache_size` is divisible by `total_no_of_cache_stores`.
    ///
    #[cfg(feature="caching")]
    pub fn set_cache_config(&mut self, total_cache_size: usize, total_no_of_cache_stores: usize) {
        assert!(
            total_cache_size % total_no_of_cache_stores == 0,
            "TOTAL CACHE SIZE MUST BE DIVISIBLE BY TOTAL NUMBER OF CACHE STORES"
        );
        let _ = unsafe { Box::from_raw(self.cache.as_ptr()) };
        let eache_cache_store_size = total_cache_size / total_no_of_cache_stores;
        self.cache = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Cache::new(
                total_no_of_cache_stores,
                eache_cache_store_size,
            ))))
        };
    }

    /// Register a route i.e. Map a url with it's corresponding handler.
    /// ## Parameters
    ///   - `url` : The url we want to map.
    ///   - `methods`: Vector of HTTP methods allowed on this route.
    ///   -`func` : Name of the route handler.
    ///
    /// Can return error if supplied method is not implimented.
    ///
    pub fn register_route(
        &mut self,
        url: &str,
        methods: Vec<&str>,
        func: RouteFunction,
    ) -> Result<(), io::Error> {
        let mut method_list: Vec<Method> = Vec::new();
        for m in methods {
            if let Some(_m_) = Method::from_string(m) {
                method_list.push(_m_);
            } else {
                error!(
                    "No http method named {}. Try using only uppercase letters like GET,POST",
                    m
                );
                return Err(io::ErrorKind::InvalidInput.into());
            }
        }

        let route = Route::new(func, method_list);
        // Routes is not a NULL pointer
        unsafe { self.routes.as_mut().add_route(url, route) };
        Ok(())
    }
    // Get the local ipv4 address.
    fn server_wl01_addr() -> Option<String> {
        let udp_socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(udps) => udps,
            Err(_) => {
                return None;
            }
        };
        let _ = match udp_socket.connect("8.8.8.8:80") {
            Ok(_) => (),
            Err(_) => {
                return None;
            }
        };
        if let Ok(local_addr) = udp_socket.local_addr() {
            if let IpAddr::V4(ipv4_addr) = local_addr.ip() {
                return Some(ipv4_addr.to_string());
            } else {
                return None;
            }
        }
        return None;
    }
    /// Set total number of threads in threadpool.
    pub fn set_total_workers(&mut self, n: usize) -> () {
        self.total_workers = n;
    }
    /// Set maximum payload size. If it exceeded in any request then automatic 413 Payload too large is sent.
    pub fn set_maximum_payload_size(&mut self, size_in_MB: usize) -> () {
        self.payload_maximum_size_in_MB = size_in_MB;
    }
    /// Set the read time out. If no packet recieved for this duration then we stop reading from stream and drop the connection.
    pub fn set_read_time_out(&mut self, time_in_secs: u8) -> () {
        self.read_time_out = Duration::from_secs(time_in_secs as u64);
    }
    /// Set the write time out. If we couldn't write on the stream for this duration we stop sending request and drop the connection.
    pub fn set_write_time_out(&mut self, time_in_secs: u8) -> () {
        self.write_time_out = Duration::from_secs(time_in_secs as u64);
    }
    /// set the maximum amount of time we keep a persistent connection alive after sending a response. Default 5 secs.
    pub fn set_keep_alive_time_out(&mut self, time_in_secs: u8) {
        self.keep_alive_time_out = Duration::from_secs(time_in_secs as u64);
    }
    /// set the maximum number of request-response cycle on a persistent connection. Default 10 request-response cycle.
    pub fn set_maximum_keep_alive_requests(&mut self, n_requests: u8) {
        self.keep_alive_max_count = n_requests;
    }
    /// Set the name of the directory where incoming files are stored. i.e. files coming in request bodies. Default name is `input_files`.
    pub fn set_incoming_files_directory_name(&mut self,directory_name:&str){
        self.file_upload_directory_name=String::from(directory_name);
    }

    /// Run the application.
    ///
    /// ## Parameters
    ///   - `host` : IP address of the API. Either It can be `127.0.0.1` then it can only be accessed from your local machine
    /// or `0.0.0.0` then It can be accessed from all devices in your LAN.
    ///   - `port` : Port on which the API listens to. If It's zero then a random available port is assigned.
    pub fn run(&mut self, host: &str, port: u16) -> () {
        //termination and interuption signal handling
        unsafe {
            let mut sig_int_act: sigaction_struct = std::mem::zeroed();
            let mut sig_term_act: sigaction_struct = std::mem::zeroed();
            let mut sig_tstp_act: sigaction_struct = std::mem::zeroed();
            sig_int_act.sa_sigaction = handle_shutdown_signal as usize;
            sig_term_act.sa_sigaction = handle_shutdown_signal as usize;
            sig_tstp_act.sa_sigaction = handle_sigtstp as usize;
            sigaddset(&mut sig_int_act.sa_mask, SIGTERM);
            sigaddset(&mut sig_int_act.sa_mask, SIGTSTP);
            sigaddset(&mut sig_term_act.sa_mask, SIGINT);
            sigaddset(&mut sig_term_act.sa_mask, SIGTSTP);
            sigaddset(&mut sig_tstp_act.sa_mask, SIGINT);
            sigaddset(&mut sig_tstp_act.sa_mask, SIGTERM);
            sig_int_act.sa_flags = SA_RESTART;
            sig_term_act.sa_flags = SA_RESTART;
            sig_tstp_act.sa_flags = SA_RESTART;
            sigaction_syscall(SIGINT, &sig_int_act, std::ptr::null_mut());
            sigaction_syscall(SIGTERM, &sig_term_act, std::ptr::null_mut());
            sigaction_syscall(SIGTSTP, &sig_tstp_act, std::ptr::null_mut());
        }
        let mut addr = String::from(host);
        addr.push(':');
        addr.push_str(&port.to_string());
        let listner = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };
        let send_buf_sz=get_send_buffer_len(listner.as_raw_fd());
        let mut app_env = AppEnv::new(host, port, &self,send_buf_sz);
        if let Ok(server_addr) = listner.local_addr() {
            if server_addr.ip().is_unspecified() {
                if let Some(wl10) = Self::server_wl01_addr() {
                    addr = wl10.clone();
                    addr.push(':');
                    addr.push_str(server_addr.port().to_string().as_str());
                    app_env.host = wl10;
                    app_env.port = server_addr.port();
                }
            } else {
                addr = String::from("127.0.0.1");
                app_env.host = addr.clone();
                addr.push(':');
                addr.push_str(server_addr.port().to_string().as_str());
                app_env.port = server_addr.port();
            }
        }
        log_info!("SERVER STARTED");
        log_info!("Listening to {}", &addr);
        log_info!("Press CTRL + C to stop the server.");
        log_info!("Process PID : {}", process::id());
        unsafe {
            PORT = app_env.port;
        }
        let pool = ThreadPool::new(self.total_workers);
        let max_keep_alive_count = app_env.keep_alive_max_count;
        let app_env_arc = Arc::new(app_env);
        for stream in listner.incoming().flatten() {
            unsafe {
                if SIG_FLAG {
                    break;
                }
            }
            let stream = Arc::new(stream);
            let app_env_cloned = Arc::clone(&app_env_arc);
            let cnt = max_keep_alive_count;
            pool.execute(move || {
                let _ = Client(stream, app_env_cloned, cnt, true);
            });
        }
    }
}
#[cfg(test)]
mod apitest {
    use super::*;
    use io::Read;
    use reqwest::{
        blocking::Client,
        header::{self, HeaderMap, HeaderValue}
    };
    use std::{collections::HashMap, fs};
    use std::thread;
    use utils::{ContentType, FileType};
    use Request::HttpRequest;
    use Response::{create_response, send_file, HttpResponse};
    enum TestResult {
        PASSED,
        FAILED(String),
        
    }
    fn json_header_path_params(
        _req: &HttpRequest,
        _path_params: HashMap<String, String>,
    ) -> HttpResponse {
        let resp_json = r#"{
        "Foo" : "Bar",
        "Dummy" : 5
        }"#;
        let mut resp = create_response(&resp_json, 200, ContentType::JSON, false).unwrap();
        for (key, val) in _req.headers.iter() {
            resp.add_header(&key, &val);
        }
        for (key, val) in _path_params.iter() {
            resp.add_header(&key, &val);
        }
        resp
    }
    fn file_download(_req:&HttpRequest,_path_params:HashMap<String,String>)->HttpResponse{
        let resp=send_file("src/test/test.jpg", None,FileType::JPEG,200,false).unwrap();
        return resp;
    }
    fn file_upload(req:&HttpRequest,_path_params:HashMap<String,String>)->HttpResponse{
        if let Some(file_path) =&req.body_location{
          let mut uploaded_file=match fs::File::open(file_path){
            Ok(f)=>f,
            Err(e)=>{
                let mut s=String::from("FAILED TO OPEN UPLOADED FILE.\n");
                s.push_str(e.to_string().as_str());
                let resp=create_response(&s, 400, ContentType::TEXT, false).unwrap();
                return resp;
            }
          };
          let mut uploaded_file_buf=Vec::<u8>::new();
          let _=match uploaded_file.read_to_end(&mut uploaded_file_buf){
            Ok(_)=>(),
            Err(e)=>{
                let mut s=String::from("FAILED TO READ UPLOADED FILE.\n");
                s.push_str(e.to_string().as_str());
                let resp=create_response(&s, 400, ContentType::TEXT, false).unwrap();
                return resp;
            }
          };
          let mut orignal_file=match fs::File::open("src/test/test.jpg"){
            Ok(f)=>f,
            Err(e)=>{
                let mut s=String::from("FAILED TO OPEN ORIGNAL FILE.\n");
                s.push_str(e.to_string().as_str());
                let resp=create_response(&s, 400, ContentType::TEXT, false).unwrap();
                return resp;
            }
          };
          let mut orignal_file_buf=Vec::<u8>::new();
          let _=match orignal_file.read_to_end(&mut orignal_file_buf){
            Ok(_)=>(),
            Err(e)=>{
                let mut s=String::from("FAILED TO READ ORIGNAL FILE.\n");
                s.push_str(e.to_string().as_str());
                let resp=create_response(&s, 400, ContentType::TEXT, false).unwrap();
                return resp;
            }
          };
          if uploaded_file_buf.eq(&orignal_file_buf){
            let resp=create_response("SUCCESS", 200, ContentType::TEXT, false).unwrap();
            return resp;
          }
          else {
            let resp=create_response("ORIGNAL FILE AND UPLOADED FILE DOESN'T MATCH", 200, ContentType::TEXT, false).unwrap();
            return resp;
          }
        }
        else {
            
            let resp=create_response("BODY LOCATION IS NONE.", 400, ContentType::TEXT, false).unwrap();
            return resp;
        }
    }
    fn run_server(){
        let mut app = RastAPI::new();
        let _ = app.register_route("/json/{id}/{name}", vec!["GET"], json_header_path_params).expect("FAILED TO REGISTER 1");
        let _ = app.register_route("/download", vec!["GET"], file_download).expect("FAILED TO REGISTER 2");
        let _=app.register_route("/upload", vec!["POST"], file_upload);
        app.run("127.0.0.1", 5000);
    }
    #[test]
    fn json_header_path_params_test() {
        let _handle1 = thread::spawn(|| {
                run_server();
        });
        thread::sleep(std::time::Duration::from_secs(1));
        let handle2 = thread::spawn(|| {
            let mut headers = header::HeaderMap::new();
            headers.insert("X-api-key", HeaderValue::from_static("abcdef12"));
            let response = match Client::new()
                .get("http://127.0.0.1:5000/json/5/rony")
                .headers(headers)
                .send(){
                    Ok(R)=>R,
                    Err(e)=>{
                        let mut s=String::from("FAILED TO SEND REQUEST. REASON :\n");
                        s.push_str(e.to_string().as_str());
                        return TestResult::FAILED(s);
                    }
                };
            let resp_json = r#"{
        "Foo" : "Bar",
        "Dummy" : 5
        }"#;
            let resp_json_vec = resp_json.as_bytes().to_vec();
            let h = response.headers();
            if !(h.get("id").map(|hv| hv.to_str().unwrap()).eq(&Some("5"))){
                return TestResult::FAILED("FAILED PATH PARAM 1".to_string());
            }
            if !(h.get("name").map(|hv| hv.to_str().unwrap()).eq(&Some("rony"))){
                return TestResult::FAILED("FAILED PATH PARAM 2".to_string());
            }
            if !(h.get("X-api-key").map(|hv| hv.to_str().unwrap()).eq(&Some("abcdef12"))){
                return TestResult::FAILED("FAILED HEADER".to_string());
            }
            let body = match response.bytes(){
                Ok(b)=>b.to_vec(),
                Err(e)=>{
                    let mut s=String::from("FAILED TO RECIEVE BYTES FROM SERVER. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            if !(body.eq(&resp_json_vec)){
                return TestResult::FAILED(String::from("BODY MISMATCH"));
            }
            TestResult::PASSED
        });
        let res=handle2.join().expect("FAILED TO JOIN");
        match res {
            TestResult::FAILED(S)=>{
                assert!(false,"{}",S);
            },
            TestResult::PASSED=>()
        };
    }
    #[test]
    fn file_download_test(){
            let _handle1 = thread::spawn(|| {
                run_server();
            });
        let _=thread::sleep(std::time::Duration::from_secs(1));
        let handle2=thread::spawn(||{
            let resp=match Client::new().get("http://127.0.0.1:5000/download").send(){
                Ok(R)=>R,
                Err(e)=>{
                    let mut s=String::from("FAILED TO SEND REQUEST. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            let mut file=match fs::File::open("src/test/test.jpg"){
                Ok(F)=>F,
                Err(e)=>{
                    let mut s=String::from("FAILED TO OPEN ORIGNAL FILE. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            let mut orignal_buf=Vec::<u8>::new();
            let _=match file.read_to_end(&mut orignal_buf){
                Ok(_)=>(),
                Err(e)=>{
                    let mut s=String::from("FAILED TO LOAD ORIGNAL FILE. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            let downloaded_buf=match resp.bytes(){
                Ok(b)=>b.to_vec(),
                Err(e)=>{
                    let mut s=String::from("FAILED TO RECIEVE BYTES FROM SERVER. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            if !(orignal_buf.eq(&downloaded_buf)){
                return TestResult::FAILED(String::from("ORIGNAL FILE AND DOWNLOADED FILE MISMATCH"));
            }
            TestResult::PASSED
        });
        let res=handle2.join().expect("FAILED TO JOIN");
        match res {
            TestResult::FAILED(S)=>{
                assert!(false,"{}",S);
            },
            TestResult::PASSED=>()
        };
    }
    #[test]
    fn file_upload_test(){
        let _handle1=thread::spawn(||{
            run_server();
        });
        let handle2=thread::spawn(||{
            let test_file=match fs::File::open("src/test/test.jpg"){
                Ok(f)=>f,
                Err(e)=>{
                    let mut s=String::from("FAILED TO OPEN TEST FILE. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            let mut headers=HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("image/jpeg"));
            let resp=match Client::new().post("http://127.0.0.1:5000/upload").headers(headers).body(test_file).send(){
                Ok(R)=>R,
                Err(e)=>{
                    let mut s=String::from("FAILED TO SEND REQUEST. REASON :\n");
                    s.push_str(e.to_string().as_str());
                    return TestResult::FAILED(s);
                }
            };
            if resp.status().as_u16()!=200_u16{
                let content=match resp.text(){
                    Ok(s)=>s,
                    Err(e)=>{
                        let mut s=String::from("FAILED TO GET RESPONSE TEXT. REASON :\n");
                        s.push_str(e.to_string().as_str());
                        return TestResult::FAILED(s);
                    }
                };
                return TestResult::FAILED(content);
            }
            TestResult::PASSED
        });
        let res=handle2.join().expect("FAILED TO JOIN");
        match res {
            TestResult::FAILED(s)=>{
                assert!(false,"{}",s);
            },
            TestResult::PASSED=>()
        }
        
    }
}
