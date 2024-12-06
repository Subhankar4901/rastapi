use super::AppEnv;
use crate::{
    error, log_response_2xx_content, log_response_2xx_file, log_response_3xx_content,
    log_response_3xx_file, log_response_4xx, log_response_5xx,
    utils::{self, ContentType, HTTPCode, Protocall},
    Request,
    Response::pre_built_resp::{
        ContentNotSupported, ContentTypeRequired, InternalServerError, InvalidContentLength,
        MethodNotAllowed, MethodNotSupported, Notfound404, PayloadTooLarge, ReaquestNotHttp,
        RequestTimeout, UTF8Error,
    },
};
use std::{collections::HashMap, net::TcpStream, sync::Arc};
use Request::HttpRequest;
/// This function handles clients. For every request this function gets to run.
pub(crate) fn Client(
    conn: Arc<TcpStream>,
    app_env: Arc<AppEnv>,
    keep_alive_count_left: u8,
    first_req: bool,
) -> () {
    let req_map = match HttpRequest::from_tcp_stream(conn.clone(), app_env.clone()) {
        Ok(_req) => _req,
        Err(e) => match e {
            utils::error::ApiError::RequestBodyNotRead(_msg) => {
                let mut resp = InternalServerError("Error happend while reading body.");
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::InvalidContentLength(_msg) => {
                let mut resp = InvalidContentLength();
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::ContentNotSupported(_msg) => {
                let mut resp = ContentNotSupported();
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::RequestTimedout(_msg) => {
                let mut resp = RequestTimeout();
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::RequestDataNotUTF8(_msg) => {
                let mut resp = UTF8Error();
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::ContentTypeRequired(_msg) => {
                let mut resp = ContentTypeRequired();
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::MethodNotAllowed(_msg) => {
                let mut resp = MethodNotAllowed(_msg.unwrap());
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::MethodNotSupported(_msg) => {
                let mut resp = MethodNotSupported(_msg.unwrap());
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::ResourceNotFound(_msg) => {
                let mut resp = Notfound404(_msg.unwrap());
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::PayloadTooLarge(_msg) => {
                let size = app_env.maximum_pay_load_in_bytes / (1024 * 1024);
                let mut resp = PayloadTooLarge(size);
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::RequestNotHttp(_msg) => {
                let msg = _msg.unwrap();
                let mut resp = ReaquestNotHttp(msg);
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::CannotWriteDataToDisk(_msg) => {
                let mut resp = InternalServerError("Internal server error.");
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            utils::error::ApiError::RequestReadError(_msg) => {
                let mut resp = InternalServerError("Error while reading request metadata.");
                resp.add_header("Host", &app_env.host);
                let _ = resp.send_response(
                    conn.clone(),
                    utils::Protocall::HTTP1_1,
                    app_env.write_time_out,
                    app_env.cache.clone(),
                    None,
                    app_env.send_buffer_size
                );
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
            _ => {
                let _ = conn.shutdown(std::net::Shutdown::Both);
                return;
            }
        },
    };
    let mut client_keep_alive = false;
    #[allow(unused_assignments)]
    let mut server_keep_alive = false;
    if first_req {
        if let Some(keep_alive_val) = req_map.headers.get("Connection") {
            if keep_alive_val.as_str().to_ascii_lowercase() == "keep-alive" {
                client_keep_alive = true;
            }
        } else {
            if req_map.protocol == Protocall::HTTP1_1 {
                client_keep_alive = true;
            }
        }
    }
    if !first_req {
        client_keep_alive = true;
    }
    let mut path_params = HashMap::new();
    let route = unsafe { app_env.routes.as_ref() }
        .get_route(&req_map.resource, Some(&mut path_params))
        .unwrap();

    let mut resp = (route.function)(&req_map, path_params);
    resp.add_header("Host", &app_env.host);

    if resp.keep_alive && first_req {
        resp.add_header("Connection", "keep-alive");
        resp.add_header(
            "Keep-Alive",
            format!(
                "timeout={}, max={}",
                app_env.keep_alive_time_out.as_secs(),
                app_env.keep_alive_max_count
            )
            .as_str(),
        );
        server_keep_alive = true;
    } else if resp.keep_alive {
        server_keep_alive = true;
    } else {
        server_keep_alive = false;
        resp.add_header("Connection", "close");
    }
    let etag_val_recieved = req_map.headers.get("If-None-Match").map(|s| s.clone());
    match resp.send_response(
        conn.clone(),
        req_map.protocol,
        app_env.write_time_out,
        app_env.cache.clone(),
        etag_val_recieved,
        app_env.send_buffer_size
    ) {
        Ok(c) => {
            let code = c;
            if code >= 200 && code < 300 {
                if resp.file_response {
                    if let Some(content_loc) = &resp.file_content_location {
                        log_response_2xx_file!(req_map.method, req_map.resource, code, content_loc);
                    }
                } else {
                    let mut contyp = "String";
                    if resp.content_type == ContentType::JSON {
                        contyp = "JSON";
                    }
                    log_response_2xx_content!(req_map.method, req_map.resource, code, contyp);
                }
            } else if code >= 300 && code < 400 {
                if resp.file_response {
                    if let Some(content_loc) = &resp.file_content_location {
                        log_response_3xx_file!(req_map.method, req_map.resource, code, content_loc);
                    }
                } else {
                    let mut contyp = "String";
                    if resp.content_type == ContentType::JSON {
                        contyp = "JSON";
                    }
                    log_response_3xx_content!(req_map.method, req_map.resource, code, contyp);
                }
            } else if code >= 400 && code < 500 {
                let msg = HTTPCode::from_u16(code).unwrap().get_msg().to_string();
                log_response_4xx!(req_map.method, req_map.resource, code, msg);
            } else {
                log_response_5xx!(req_map.method, req_map.resource, code);
            }
            if keep_alive_count_left > 0 && client_keep_alive && server_keep_alive {
                Client(
                    Arc::clone(&conn),
                    Arc::clone(&app_env),
                    keep_alive_count_left - 1,
                    false,
                );
            }
            let _ = conn.shutdown(std::net::Shutdown::Both);
            return;
        }
        Err(e) => {
            error!("{}", e);
            let _ = conn.shutdown(std::net::Shutdown::Both);
            return;
        }
    }
}
