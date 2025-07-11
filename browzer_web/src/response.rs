use chrono;
use crate::utils;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: utils::HttpStatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub cookies: HashMap<String, utils::Cookie>,
}

impl Default for Response {
    fn default() -> Self {
        return Response {
            status_code: utils::HttpStatusCode::OK,
            headers: HashMap::new(),
            body: String::from(""),
            cookies: HashMap::new(),
        };
    }
}

impl Response {
    pub fn new(status_code: utils::HttpStatusCode, body: String) -> Response {
        return Response {
            status_code,
            headers: HashMap::new(),
            body,
            cookies: HashMap::new(),
        };
    }

    // This function convert the `Response` struct into a string to be sent as bytes by setting the status_code
    // number, status_code text, and content-length in the `Status Line`, setting headers
    // to the response string by looping over `headers` field in the Response struct and looping
    // over the `cookies` field in the Response struct, and then finally adding a blank line
    // followed by the body of the response to the response string
    pub fn to_string(&self) -> String {
        let status_code = &self.status_code.code();
        let mut response = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n",
            status_code.1,
            status_code.0,
            &self.body.len(),
        );
        for (key, value) in &self.headers {
            response.push_str(&format! {"{}: {}\r\n",key,value});
        }

        // parse cookies hashmap and append it to the response string
        for cookie in self.cookies.values() {
            let mut cookie_string = format!("{}={}", cookie.name, cookie.value);

            if let Some(ref path) = cookie.path {
                cookie_string.push_str(&format!("; Path={}", path));
            }

            if let Some(ref domain) = cookie.domain {
                cookie_string.push_str(&format!("; Domain={}", domain));
            }

            if let Some(expires) = cookie.expires {
                let datetime = chrono::DateTime::<chrono::Utc>::from(expires);
                let formatted_time = datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
                cookie_string.push_str(&format!("; Expires={}", formatted_time));
            }

            if let Some(max_age) = cookie.max_age {
                cookie_string.push_str(&format!("; Max-Age={}", max_age));
            }

            if cookie.secure {
                cookie_string.push_str("; Secure");
            }

            if cookie.http_only {
                cookie_string.push_str("; HttpOnly");
            }

            response.push_str(&format!("Set-Cookie: {}\r\n", cookie_string));
        }

        response.push_str("\r\n");
        response.push_str(&self.body);
        return response;
    }
}
