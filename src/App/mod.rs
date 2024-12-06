#![allow(dead_code)]
pub(crate) mod client;
use crate::{cache::Cache, utils, RastAPI, Request, Response};
use regex;
use regex::Regex;
use std::{collections::HashMap, path::PathBuf, ptr::NonNull, time::Duration};
use utils::Method;
use Request::HttpRequest;
use Response::HttpResponse;
/// Signature of route handlers. i.e. functions that get called on every request.
pub(crate) type RouteFunction = fn(&HttpRequest, HashMap<String, String>) -> HttpResponse;
/// Structure to store router function and allowed methods on that route.
#[derive(Debug)]
pub(crate) struct Route {
    pub(crate) function: RouteFunction,
    pub(crate) methods: Vec<Method>,
}
impl Route {
    pub(crate) fn new(func: RouteFunction, mut methods: Vec<Method>) -> Self {
        if methods.is_empty() {
            methods.push(Method::GET);
        }
        Self {
            function: func,
            methods: methods,
        }
    }
}
/// Main router of our REST API. It maps urls to their corresponding handlers.
pub(crate) struct URLRouter {
    pub(crate) router: HashMap<String, Route>,
}

impl URLRouter {
    pub(crate) fn new() -> Self {
        Self {
            router: HashMap::new(),
        }
    }
    /// Get a route for a specific URL. It also determines the path parameters.
    pub(crate) fn get_route(
        &self,
        url: &str,
        path_params_opt: Option<&mut HashMap<String, String>>,
    ) -> Option<&Route> {
        for (key, val) in self.router.iter() {
            let re = Regex::new(key).unwrap();
            if re.is_match(url) {
                if let Some(path_params) = path_params_opt {
                    if let Some(captures) = re.captures(url) {
                        for name in re.capture_names().flatten() {
                            if let Some(val) = captures.name(name) {
                                path_params.insert(name.to_string(), val.as_str().to_string());
                            }
                        }
                    }
                }

                return Some(val);
            }
        }
        None
    }
    /// Register a route.
    pub(crate) fn add_route(&mut self, url: &str, route: Route) -> () {
        let re = Regex::new(r#"\{([a-zA-Z0-9_]+)\}"#).unwrap();
        let regex_pattern = re.replace_all(url, "(?P<$1>[ -~]+)").to_string();
        let mut final_pattern = String::from("^");
        final_pattern.push_str(&regex_pattern);
        final_pattern.push('$');
        let _res = self.router.insert(final_pattern, route);
    }
    /// Remove a already registered route.
    pub(crate) fn remove_route(&mut self, url: &str) -> () {
        let re = Regex::new(r#"\{([a-zA-Z0-9_]+)\}"#).unwrap();
        let regex_pattern = re.replace_all(url, "(?P<$1>[ -~]+)").to_string();
        let mut final_pattern = String::from("^");
        final_pattern.push_str(&regex_pattern);
        final_pattern.push('$');
        self.router.remove(&final_pattern);
    }
}
/// Some data related to the current app and other configurations which gets passed to the job handlers for every request.
pub(crate) struct AppEnv {
    pub host: String,
    pub port: u16,
    pub cache: NonNull<Cache<PathBuf>>,
    pub routes: NonNull<URLRouter>,
    pub maximum_pay_load_in_bytes: usize,
    pub read_time_out: Duration,
    pub write_time_out: Duration,
    pub keep_alive_time_out: Duration,
    pub keep_alive_max_count: u8,
    pub incoming_file_directory: String,
    pub send_buffer_size:usize
}
impl AppEnv {
    pub fn new(host: &str, port: u16, app: &RastAPI,send_buf_size:usize) -> Self {
        Self {
            host: host.to_string(),
            port,
            cache: app.cache,
            routes: app.routes,
            maximum_pay_load_in_bytes: app.payload_maximum_size_in_MB * 1024 * 1024,
            read_time_out: app.read_time_out,
            write_time_out: app.write_time_out,
            keep_alive_time_out: app.keep_alive_time_out,
            keep_alive_max_count: app.keep_alive_max_count,
            incoming_file_directory: app.file_upload_directory_name.clone(),
            send_buffer_size:send_buf_size
        }
    }
}
unsafe impl Send for AppEnv {}
unsafe impl Sync for AppEnv {}
