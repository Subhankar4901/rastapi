#![allow(dead_code)]
//! # Utils
//! This is a utility module that helps by providing some utility functions,enums and structs.
use error::ApiError;
pub(crate) mod error;
pub mod load_env;
pub(crate) mod threadpool;
/// FileType is an enum that indicates the type of a file. It helps in sending file responses.
/// If file type is unknown then we send the file as a binary with *Content-Type : application/octet-stream*.
/// ## Example
/// ```no_run
/// use rastapi::utils::FileType;
/// let file_type=FileType::PNG; // PNG image type file.
/// let unknown_type=FileType::UNKNOWN; // For unknown types.
/// ```
#[derive(Clone, Copy)]
pub enum FileType {
    TEXT,
    JSON,
    XML,
    YAML,
    PNG,
    JPEG,
    SVG,
    WEBP,
    CSV,
    XLSX,
    PDF,
    PPTX,
    DOCX,
    MP3,
    WAV,
    MP4,
    ZIP,
    GZIP,
    EXE,
    UNKNOWN,
}
impl FileType {
    pub(crate) fn extension(&self) -> &str {
        let ext = match self {
            FileType::TEXT => ".txt",
            FileType::JSON => ".json",
            FileType::XML => ".xml",
            FileType::YAML => ".yaml",
            FileType::PNG => ".png",
            FileType::JPEG => ".jpeg",
            FileType::SVG => ".svg",
            FileType::WEBP => ".webp",
            FileType::CSV => ".csv",
            FileType::XLSX => ".xlsx",
            FileType::PDF => ".pdf",
            FileType::PPTX => ".pptx",
            FileType::DOCX => ".docx",
            FileType::MP3 => ".mp3",
            FileType::WAV => ".wav",
            FileType::MP4 => ".mp4",
            FileType::ZIP => ".zip",
            FileType::GZIP => ".gz",
            FileType::EXE => ".exe",
            FileType::UNKNOWN => "",
        };
        ext
    }
    pub(crate) fn to_content_type(&self) -> ContentType {
        match self {
            FileType::TEXT => ContentType::TEXT,
            FileType::JSON => ContentType::JSON,
            FileType::XML => ContentType::XML,
            FileType::YAML => ContentType::YAML,
            FileType::PNG => ContentType::PNG,
            FileType::JPEG => ContentType::JPEG,
            FileType::SVG => ContentType::SVG,
            FileType::WEBP => ContentType::WEBP,
            FileType::CSV => ContentType::CSV,
            FileType::XLSX => ContentType::XLSX,
            FileType::PDF => ContentType::PDF,
            FileType::PPTX => ContentType::PPTX,
            FileType::DOCX => ContentType::DOCX,
            FileType::MP3 => ContentType::MP3,
            FileType::WAV => ContentType::WAV,
            FileType::MP4 => ContentType::MP4,
            FileType::ZIP => ContentType::ZIP,
            FileType::GZIP => ContentType::GZIP,
            FileType::EXE => ContentType::EXE,
            FileType::UNKNOWN => ContentType::UNKNOWN,
        }
    }
}
/// Content type is an enum that indicates the type of content we are sending or recieving.
/// It can be Unknown if Content-Type is *application/octet-stram*.
///
/// ## Example
/// ```no_run
/// use rastapi::utils::ContentType;
///  let json_content_type=ContentType::JSON;
/// ```
#[derive(Clone, Copy, PartialEq)]
pub enum ContentType {
    TEXT,
    JSON,
    XML,
    YAML,
    PNG,
    JPEG,
    SVG,
    WEBP,
    CSV,
    XLSX,
    PDF,
    PPTX,
    DOCX,
    MP3,
    WAV,
    MP4,
    ZIP,
    GZIP,
    EXE,
    UNKNOWN,
}
impl ContentType {
    pub(crate) fn get_content_type_header(&self) -> &str {
        match self {
            ContentType::TEXT => "Content-Type: text/plain\r\n",
            ContentType::JSON => "Content-Type: application/json\r\n",
            ContentType::XML => "Content-Type: application/xml\r\n",
            ContentType::YAML => "Content-Type: application/x-yaml\r\n",
            ContentType::PNG => "Content-Type: image/png\r\n",
            ContentType::JPEG => "Content-Type: image/jpeg\r\n",
            ContentType::SVG => "Content-Type: image/svg+xml\r\n",
            ContentType::WEBP => "Content-Type: image/webp\r\n",
            ContentType::CSV => "Content-Type: text/csv\r\n",
            ContentType::XLSX => "Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\r\n",
            ContentType::PDF => "Content-Type: application/pdf\r\n",
            ContentType::PPTX => "Content-Type: application/vnd.openxmlformats-officedocument.presentationml.presentation\r\n",
            ContentType::DOCX => "Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n",
            ContentType::MP3 => "Content-Type: audio/mpeg\r\n",
            ContentType::WAV => "Content-Type: audio/wav\r\n",
            ContentType::MP4 => "Content-Type: video/mp4\r\n",
            ContentType::ZIP => "Content-Type: application/zip\r\n",
            ContentType::GZIP => "Content-Type: application/gzip\r\n",
            ContentType::EXE => "Content-Type: application/x-msdownload\r\n",
            ContentType::UNKNOWN => "Content-Type: application/octet-stream\r\n",
        }
    }
    pub(crate) fn from_header(header: &str) -> Result<Self, ApiError> {
        let content_type = header.trim().to_lowercase();
        let res = match content_type.as_str() {
            "text/plain" => ContentType::TEXT,
            "application/json" => ContentType::JSON,
            "application/xml" => ContentType::XML,
            "application/x-yaml" => ContentType::YAML,
            "image/png" => ContentType::PNG,
            "image/jpeg" => ContentType::JPEG,
            "image/svg+xml" => ContentType::SVG,
            "image/webp" => ContentType::WEBP,
            "text/csv" => ContentType::CSV,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                ContentType::XLSX
            }
            "application/pdf" => ContentType::PDF,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
                ContentType::PPTX
            }
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                ContentType::DOCX
            }
            "audio/mpeg" => ContentType::MP3,
            "audio/wav" => ContentType::WAV,
            "video/mp4" => ContentType::MP4,
            "application/zip" => ContentType::ZIP,
            "application/gzip" => ContentType::GZIP,
            "application/x-msdownload" => ContentType::EXE,
            "application/octet-stream" => ContentType::UNKNOWN,
            _ => {
                return Err(ApiError::ContentNotSupported(None));
            }
        };
        Ok(res)
    }
    pub(crate) fn to_file_type(&self) -> FileType {
        match self {
            ContentType::TEXT => FileType::TEXT,
            ContentType::JSON => FileType::JSON,
            ContentType::XML => FileType::XML,
            ContentType::YAML => FileType::YAML,
            ContentType::PNG => FileType::PNG,
            ContentType::JPEG => FileType::JPEG,
            ContentType::SVG => FileType::SVG,
            ContentType::WEBP => FileType::WEBP,
            ContentType::CSV => FileType::CSV,
            ContentType::XLSX => FileType::XLSX,
            ContentType::PDF => FileType::PDF,
            ContentType::PPTX => FileType::PPTX,
            ContentType::DOCX => FileType::DOCX,
            ContentType::MP3 => FileType::MP3,
            ContentType::WAV => FileType::WAV,
            ContentType::MP4 => FileType::MP4,
            ContentType::ZIP => FileType::ZIP,
            ContentType::GZIP => FileType::GZIP,
            ContentType::EXE => FileType::EXE,
            ContentType::UNKNOWN => FileType::UNKNOWN,
        }
    }
}
/// HTTP Code is an enum that indicates HTTP Response code.
#[derive(Clone, Copy)]
pub(crate) enum HTTPCode {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,

    // 2xx Success
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,

    // 3xx Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Errors
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418, // Fun Easter Egg from RFC 2324
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // 5xx Server Errors
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HTTPCode {
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            100 => Some(Self::Continue),
            101 => Some(Self::SwitchingProtocols),
            102 => Some(Self::Processing),
            200 => Some(Self::OK),
            201 => Some(Self::Created),
            202 => Some(Self::Accepted),
            203 => Some(Self::NonAuthoritativeInformation),
            204 => Some(Self::NoContent),
            205 => Some(Self::ResetContent),
            206 => Some(Self::PartialContent),
            207 => Some(Self::MultiStatus),
            208 => Some(Self::AlreadyReported),
            226 => Some(Self::IMUsed),
            300 => Some(Self::MultipleChoices),
            301 => Some(Self::MovedPermanently),
            302 => Some(Self::Found),
            303 => Some(Self::SeeOther),
            304 => Some(Self::NotModified),
            305 => Some(Self::UseProxy),
            307 => Some(Self::TemporaryRedirect),
            308 => Some(Self::PermanentRedirect),
            400 => Some(Self::BadRequest),
            401 => Some(Self::Unauthorized),
            402 => Some(Self::PaymentRequired),
            403 => Some(Self::Forbidden),
            404 => Some(Self::NotFound),
            405 => Some(Self::MethodNotAllowed),
            406 => Some(Self::NotAcceptable),
            407 => Some(Self::ProxyAuthenticationRequired),
            408 => Some(Self::RequestTimeout),
            409 => Some(Self::Conflict),
            410 => Some(Self::Gone),
            411 => Some(Self::LengthRequired),
            412 => Some(Self::PreconditionFailed),
            413 => Some(Self::PayloadTooLarge),
            414 => Some(Self::URITooLong),
            415 => Some(Self::UnsupportedMediaType),
            416 => Some(Self::RangeNotSatisfiable),
            417 => Some(Self::ExpectationFailed),
            418 => Some(Self::ImATeapot),
            421 => Some(Self::MisdirectedRequest),
            422 => Some(Self::UnprocessableEntity),
            423 => Some(Self::Locked),
            424 => Some(Self::FailedDependency),
            426 => Some(Self::UpgradeRequired),
            428 => Some(Self::PreconditionRequired),
            429 => Some(Self::TooManyRequests),
            431 => Some(Self::RequestHeaderFieldsTooLarge),
            451 => Some(Self::UnavailableForLegalReasons),
            500 => Some(Self::InternalServerError),
            501 => Some(Self::NotImplemented),
            502 => Some(Self::BadGateway),
            503 => Some(Self::ServiceUnavailable),
            504 => Some(Self::GatewayTimeout),
            505 => Some(Self::HTTPVersionNotSupported),
            506 => Some(Self::VariantAlsoNegotiates),
            507 => Some(Self::InsufficientStorage),
            508 => Some(Self::LoopDetected),
            510 => Some(Self::NotExtended),
            511 => Some(Self::NetworkAuthenticationRequired),
            _ => None,
        }
    }
    pub fn get_msg(&self) -> &str {
        match self {
            // 1xx Informational
            HTTPCode::Continue => "Continue",
            HTTPCode::SwitchingProtocols => "Switching Protocols",
            HTTPCode::Processing => "Processing",

            // 2xx Success
            HTTPCode::OK => "OK",
            HTTPCode::Created => "Created",
            HTTPCode::Accepted => "Accepted",
            HTTPCode::NonAuthoritativeInformation => "Non-Authoritative Information",
            HTTPCode::NoContent => "No Content",
            HTTPCode::ResetContent => "Reset Content",
            HTTPCode::PartialContent => "Partial Content",
            HTTPCode::MultiStatus => "Multi-Status",
            HTTPCode::AlreadyReported => "Already Reported",
            HTTPCode::IMUsed => "IM Used",

            // 3xx Redirection
            HTTPCode::MultipleChoices => "Multiple Choices",
            HTTPCode::MovedPermanently => "Moved Permanently",
            HTTPCode::Found => "Found",
            HTTPCode::SeeOther => "See Other",
            HTTPCode::NotModified => "Not Modified",
            HTTPCode::UseProxy => "Use Proxy",
            HTTPCode::TemporaryRedirect => "Temporary Redirect",
            HTTPCode::PermanentRedirect => "Permanent Redirect",

            // 4xx Client Errors
            HTTPCode::BadRequest => "Bad Request",
            HTTPCode::Unauthorized => "Unauthorized",
            HTTPCode::PaymentRequired => "Payment Required",
            HTTPCode::Forbidden => "Forbidden",
            HTTPCode::NotFound => "Not Found",
            HTTPCode::MethodNotAllowed => "Method Not Allowed",
            HTTPCode::NotAcceptable => "Not Acceptable",
            HTTPCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            HTTPCode::RequestTimeout => "Request Timeout",
            HTTPCode::Conflict => "Conflict",
            HTTPCode::Gone => "Gone",
            HTTPCode::LengthRequired => "Length Required",
            HTTPCode::PreconditionFailed => "Precondition Failed",
            HTTPCode::PayloadTooLarge => "Payload Too Large",
            HTTPCode::URITooLong => "URI Too Long",
            HTTPCode::UnsupportedMediaType => "Unsupported Media Type",
            HTTPCode::RangeNotSatisfiable => "Range Not Satisfiable",
            HTTPCode::ExpectationFailed => "Expectation Failed",
            HTTPCode::ImATeapot => "I'm a teapot",
            HTTPCode::MisdirectedRequest => "Misdirected Request",
            HTTPCode::UnprocessableEntity => "Unprocessable Entity",
            HTTPCode::Locked => "Locked",
            HTTPCode::FailedDependency => "Failed Dependency",
            HTTPCode::UpgradeRequired => "Upgrade Required",
            HTTPCode::PreconditionRequired => "Precondition Required",
            HTTPCode::TooManyRequests => "Too Many Requests",
            HTTPCode::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            HTTPCode::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            // 5xx Server Errors
            HTTPCode::InternalServerError => "Internal Server Error",
            HTTPCode::NotImplemented => "Not Implemented",
            HTTPCode::BadGateway => "Bad Gateway",
            HTTPCode::ServiceUnavailable => "Service Unavailable",
            HTTPCode::GatewayTimeout => "Gateway Timeout",
            HTTPCode::HTTPVersionNotSupported => "HTTP Version Not Supported",
            HTTPCode::VariantAlsoNegotiates => "Variant Also Negotiates",
            HTTPCode::InsufficientStorage => "Insufficient Storage",
            HTTPCode::LoopDetected => "Loop Detected",
            HTTPCode::NotExtended => "Not Extended",
            HTTPCode::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }
}
// Method is an enum that indicates HTTP methods.
#[derive(PartialEq, Debug)]
pub(crate) enum Method {
    GET,
    POST,
    UPDATE,
    PUT,
    PATCH,
    DELETE,
}
impl Method {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "UPDATE" => Some(Self::UPDATE),
            "PUT" => Some(Self::PUT),
            "PATCH" => Some(Self::PATCH),
            "DELETE" => Some(Self::DELETE),
            _ => None,
        }
    }
}
/// Protocol is an enum that indicates the protocol used by our clients to send request.
/// Currently we only support two protocols, *HTTP/1.0* and *HTTP/1.1*.
#[derive(PartialEq, Eq)]
pub enum Protocall {
    HTTP1_0,
    HTTP1_1,
}
impl Protocall {
    pub fn from_str(proto: &str) -> Option<Self> {
        match proto {
            "HTTP/1.0" => Some(Self::HTTP1_0),
            "HTTP/1.1" => Some(Self::HTTP1_1),
            _ => None,
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            Self::HTTP1_0 => "HTTP/1.0",
            Self::HTTP1_1 => "HTTP/1.1",
        }
    }
}
