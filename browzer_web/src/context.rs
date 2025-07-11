use serde_urlencoded;
use crate::{request, response, utils};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    pub request: request::Request,
    pub response: response::Response,
    pub params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl Context {
    pub fn new(request: request::Request) -> Context {
        return Context {
            request,
            response: response::Response::default(),
            params: HashMap::new(),
            query_params: HashMap::new(),
        };
    }

    pub fn send_string(
        &mut self,
        status_code: utils::HttpStatusCode,
        input: &str,
    ) -> response::Response {
        let res = &mut self.response;
        res.status_code = status_code;
        res.body = input.to_string();
        res.clone()
    }

    pub fn redirect(
        &mut self,
        status_code: utils::HttpStatusCode,
        route: &str,
    ) -> response::Response {
        let res = &mut self.response;
        res.headers
            .insert("Location".to_string(), route.to_string());
        res.status_code = status_code;
        res.clone()
    }

    pub fn form_value(&mut self, key: &str) -> String {
        match self.request.headers.get("Content-Type") {
            Some(content_type) => content_type,
            None => return String::from(""),
        };
        match serde_urlencoded::from_str::<HashMap<String, String>>(match &self.request.body {
            Some(body) => match std::str::from_utf8(body.trim().as_bytes()) {
                Ok(body_str) => body_str.trim(),
                Err(_) => return String::from(""),
            },
            None => return String::from(""),
        }) {
            Ok(data) => {
                match data.get(key) {
                    Some(value) => {
                        return value.to_string();
                    }
                    None => {
                        return String::from("");
                    }
                };
            }
            Err(_) => return String::from(""),
        };
    }
}
