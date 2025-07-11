use std::time;
use crate::error;

pub mod thread_pool;

// If there is a route defined as `/menu/items/`, a person would probably not want to add the
// slash at the end everytime they are visiting this path, so this function removes the slashes at
// the end from such paths making it easier and simpler for both the end user and developer
pub fn format_path_by_slashes(mut path: String) -> Result<String, error::WebRouterError> {
    if path.trim().len() == 0 && path.trim() == "" {
        path = "/".to_string();
    }
    match path.chars().nth(path.len() - 1) {
        Some(last_char) => {
            if last_char == '/' {
                path.pop();
            }
        }
        None => {
            return Err(error::WebRouterError::PathFormatError(
                "Failed to format path by slashes".to_string(),
            ));
        }
    }
    path = path.replace("/?", "?");
    return Ok(path);
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    DELETE,
}
impl HttpMethod {
    pub fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum HttpStatusCode {
    OK,
    Created,
    Accepted,
    NoContent,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
}
impl HttpStatusCode {
    pub fn code(&self) -> (&str, u16) {
        match self {
            HttpStatusCode::OK => ("OK", 200),
            HttpStatusCode::Created => ("Created", 201),
            HttpStatusCode::Accepted => ("Accepted", 202),
            HttpStatusCode::NoContent => ("NoContent", 204),
            HttpStatusCode::MovedPermanently => ("Moved Permanently", 301),
            HttpStatusCode::Found => ("Found", 302),
            HttpStatusCode::SeeOther => ("See Other", 303),
            HttpStatusCode::NotModified => ("Not Modified", 304),
            HttpStatusCode::BadRequest => ("Bad Request", 400),
            HttpStatusCode::Unauthorized => ("Unauthorized", 401),
            HttpStatusCode::Forbidden => ("Forbidden", 403),
            HttpStatusCode::NotFound => ("Not Found", 404),
            HttpStatusCode::MethodNotAllowed => ("Method Not Allowed", 405),
            HttpStatusCode::InternalServerError => ("Internal Server Error", 500),
            HttpStatusCode::NotImplemented => ("Not Implemented", 501),
            HttpStatusCode::BadGateway => ("Bad Gateway", 502),
            HttpStatusCode::ServiceUnavailable => ("Service Unavailable", 503),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expires: Option<time::SystemTime>,
    pub raw_expires: Option<String>,
    pub max_age: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub raw: Option<String>,
}
impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        return Cookie {
            name: name.to_string(),
            value: value.to_string(),
            ..Default::default()
        };
    }
}
impl Default for Cookie {
    fn default() -> Self {
        return Cookie {
            name: "".to_string(),
            value: "".to_string(),
            path: None,
            domain: None,
            expires: None,
            raw_expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            raw: None,
        };
    }
}
