use std::fs::ReadDir;

use http::StatusCode;
use wry::RequestAsyncResponder;

pub struct Response {
    /// ステータスコード
    pub code: StatusCode,
    /// MIME タイプ
    pub mime: String,
    /// レスポンスボディ
    pub body: Vec<u8>,
}

impl<T: Into<Response>> Into<Response> for Option<T> {
    fn into(self) -> Response {
        match self {
            Some(result) => result.into(),
            None => ().into(),
        }
    }
}
impl<T: Into<Response>> Into<Response> for Result<T, std::io::Error> {
    fn into(self) -> Response {
        match self {
            Ok(result) => result.into(),
            Err(error) => error.into(),
        }
    }
}
impl Into<Response> for std::io::Error {
    fn into(self) -> Response {
        match self.kind() {
            std::io::ErrorKind::NotFound => StatusCode::NOT_FOUND.into(),
            std::io::ErrorKind::AlreadyExists => StatusCode::CONFLICT.into(),
            std::io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN.into(),
            std::io::ErrorKind::InvalidData => StatusCode::BAD_REQUEST.into(),
            _ => {
                log::error!("An error occured: {}", self);
                StatusCode::INTERNAL_SERVER_ERROR.into()
            }
        }
    }
}
impl Into<Response> for StatusCode {
    fn into(self) -> Response {
        Response {
            code: self,
            mime: "text/plain".to_owned(),
            body: if let Some(reason) = self.canonical_reason() {
                format!("{} {}", self.as_str(), reason)
            } else {
                self.as_str().to_owned()
            }.as_bytes().to_vec(),
        }
    }
}

impl Into<Response> for () {
    fn into(self) -> Response {
        Response {
            code: StatusCode::NO_CONTENT,
            mime: "text/plain".to_owned(),
            body: vec![],
        }
    }
}
impl Into<Response> for Vec<u8> {
    fn into(self) -> Response {
        Response {
            code: StatusCode::OK,
            mime: "application/octet-stream".to_owned(),
            body: self,
        }
    }
}
impl Into<Response> for String {
    fn into(self) -> Response {
        Response {
            code: StatusCode::OK,
            mime: "text/plain".to_owned(),
            body: self.as_bytes().to_vec(),
        }
    }
}

impl Into<Response> for ReadDir {
    fn into(self) -> Response {
        let mut vec = vec![];
        for dir in self {
            if let Ok(dir) = dir {
                let file_name = dir.file_name();
                if let Some(str) = file_name.as_os_str().to_str() {
                    vec.push(str.to_owned());
                }
            }
        }
        Response {
            code: StatusCode::OK,
            mime: "text/plain".to_owned(),
            body: vec.join("\n").as_bytes().to_vec()
        }
    }
}

pub trait Responder {
    /// レスポンスを返します。
    fn respond_with<T: Into<Response>>(self, response: T);
}
impl Responder for RequestAsyncResponder {
    fn respond_with<T: Into<Response>>(self, response: T) {
        let response: Response = response.into();

        let mut inner_response = http::Response::default();
        *inner_response.status_mut() = response.code;
        inner_response.headers_mut().append("Content-Type", http::HeaderValue::from_str(&response.mime).unwrap());
        inner_response.headers_mut().append("Content-Length", http::HeaderValue::from_str(&response.body.len().to_string()).unwrap());
        *inner_response.body_mut() = response.body;

        self.respond(inner_response);
    }
}
