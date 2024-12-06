use std::borrow::Cow;

use super::{create_response, HttpResponse};
use crate::utils::ContentType;
pub fn Notfound404(msg: Cow<str>) -> HttpResponse {
    let resp = create_response(&msg, 404, ContentType::TEXT, false).unwrap();
    return resp;
}
pub fn InvalidContentLength() -> HttpResponse {
    let resp = create_response(
        "Provide a valid content length header.",
        411,
        ContentType::TEXT,
        false,
    )
    .unwrap();
    return resp;
}
pub fn ContentTypeRequired() -> HttpResponse {
    let resp = create_response(
        "Provide a content type header with post requests.",
        400,
        ContentType::TEXT,
        false,
    )
    .unwrap();
    return resp;
}
pub fn ContentNotSupported() -> HttpResponse {
    let resp = create_response(
        "Unsupported content/payload type.",
        415,
        ContentType::TEXT,
        false,
    )
    .unwrap();
    return resp;
}
pub fn MethodNotAllowed(msg: Cow<str>) -> HttpResponse {
    let resp = create_response(&msg, 405, ContentType::TEXT, false).unwrap();
    return resp;
}
pub fn MethodNotSupported(msg: Cow<str>) -> HttpResponse {
    let resp = create_response(&msg, 405, ContentType::TEXT, false).unwrap();
    return resp;
}
pub fn RequestTimeout() -> HttpResponse {
    let resp = create_response("Request timed out", 408, ContentType::TEXT, false).unwrap();
    return resp;
}
#[allow(dead_code)]
pub fn PayloadTooLarge(size: usize) -> HttpResponse {
    let resp = create_response(
        &format!("Payload too large. Size must be less than {} MB", size),
        413,
        ContentType::TEXT,
        false,
    )
    .unwrap();
    return resp;
}
pub fn UTF8Error() -> HttpResponse {
    let resp = create_response(
        "Request message metadata should be UTF-8 encoding complient.",
        413,
        ContentType::TEXT,
        false,
    )
    .unwrap();
    return resp;
}
pub fn ReaquestNotHttp(msg: Cow<str>) -> HttpResponse {
    let resp = create_response(&msg, 413, ContentType::TEXT, false).unwrap();
    return resp;
}
pub fn InternalServerError(msg: &str) -> HttpResponse {
    let resp = create_response(msg, 500, ContentType::TEXT, false).unwrap();
    return resp;
}
