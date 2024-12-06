use std::{borrow::Cow, fmt};
#[derive(Debug)]
pub enum ApiError<'a> {
    ClientNotFound(Option<Cow<'a, str>>),
    RequestReadError(Option<Cow<'a, str>>),
    RequestDataNotUTF8(Option<Cow<'a, str>>),
    RequestNotHttp(Option<Cow<'a, str>>),
    InvalidHeader(Option<Cow<'a, str>>),
    ContentNotSupported(Option<Cow<'a, str>>),
    CannotWriteDataToDisk(Option<Cow<'a, str>>),
    RequestBodyNotRead(Option<Cow<'a, str>>),
    InvalidContentLength(Option<Cow<'a, str>>),
    ContentTypeRequired(Option<Cow<'a, str>>),
    MethodNotAllowed(Option<Cow<'a, str>>),
    MethodNotSupported(Option<Cow<'a, str>>),
    ResourceNotFound(Option<Cow<'a, str>>),
    PayloadTooLarge(Option<Cow<'a, str>>),
    RequestTimedout(Option<Cow<'a, str>>),
    ClientDisconnected(Option<Cow<'a, str>>),
}
impl<'a> fmt::Display for ApiError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::ClientNotFound(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Client Socket Adress not found in tcp stream.")
                }
            }
            ApiError::RequestDataNotUTF8(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Request string data is not UTF-8 complient")
                }
            }
            ApiError::RequestReadError(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Couldn't read request.")
                }
            }
            ApiError::RequestNotHttp(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Request protocol is not HTTP/1.x")
                }
            }
            ApiError::InvalidHeader(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Invalid header value.")
                }
            }
            ApiError::ContentNotSupported(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Content-Type not supported.")
                }
            }
            ApiError::CannotWriteDataToDisk(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Cannot Write data to a file on disk.")
                }
            }
            ApiError::RequestBodyNotRead(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Cannot Read body completly.")
                }
            }
            ApiError::InvalidContentLength(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Invalid content length header.")
                }
            }
            Self::ContentTypeRequired(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Content type required.")
                }
            }
            Self::MethodNotAllowed(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Method Not allowed.")
                }
            }
            Self::ResourceNotFound(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Resource Not found.")
                }
            }
            Self::MethodNotSupported(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Method Not supported.")
                }
            }
            Self::PayloadTooLarge(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Payload Too large.")
                }
            }
            Self::RequestTimedout(msg) => {
                if let Some(err_msg) = msg {
                    write!(f, "{}", err_msg)
                } else {
                    write!(f, "Request read time out reached.")
                }
            }
            Self::ClientDisconnected(msg) => {
                if let Some(_msg) = msg {
                    writeln!(f, "{}", _msg)
                } else {
                    writeln!(f, "Client disconnected prematurely.")
                }
            }
        }
    }
}
