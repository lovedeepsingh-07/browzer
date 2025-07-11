use crate::{context, error, request, response, utils};
use std::{collections::HashMap, fmt};

pub struct WebRouter {
    // HashMap< --path-- ,HashMap< --method-- , RouteHandlerFunction>>
    pub routes: HashMap<
        String,
        HashMap<
            String,
            Box<dyn Fn(context::Context) -> response::Response + 'static + Send + Sync>,
        >,
    >,
    pub middlewares: Vec<Box<dyn Fn(context::Context) -> context::Context + 'static + Send + Sync>>,
}

impl fmt::Debug for WebRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebRouter")
            .field("routes", &"HashMap<String, HashMap<String, Box<dyn Fn(context::Context) -> response::Response + Send + Sync + 'static>>>")
            .field("middlewares", &"Vec<Box<dyn Fn(context::Context) -> context::Context + 'static + Send + Sync>>")
            .finish()
    }
}

impl WebRouter {
    pub fn new() -> WebRouter {
        return WebRouter {
            routes: HashMap::new(),
            middlewares: vec![],
        };
    }

    pub fn add<F>(
        &mut self,
        mut path: String,
        method: utils::HttpMethod,
        handler: F,
    ) -> Result<(), error::WebRouterError>
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        path = match utils::format_path_by_slashes(path) {
            Ok(formatted_path) => formatted_path,
            Err(e) => {
                return Err(e);
            }
        };
        self.routes
            .entry(path.to_string())
            .or_insert_with(HashMap::new)
            .insert(method.to_string(), Box::new(handler));
        return Ok(());
    }

    pub fn add_middleware<F>(&mut self, middleware_func: F)
    where
        F: Fn(context::Context) -> context::Context + 'static + Send + Sync,
    {
        self.middlewares.push(Box::new(middleware_func));
    }

    // This function works in two parts:
    // 1. It applies all the middlewares from the `middlewares` vector
    // 2. handle response generation from request by first getting all the user-registered routes
    // which match the request's path(it will be hashmap) from `routes` hashmap, then using that
    // hashmap to get the route which matches request's method and then finaly using that route's
    // handler function to generate the response for the request by providing a new `Context` with
    // the request as input to the handler function
    pub fn handle_request(
        &self,
        mut request: request::Request,
    ) -> Result<response::Response, error::WebRouterError> {
        // format request path by slashes
        request.path = match utils::format_path_by_slashes(request.path) {
            Ok(formatted_path) => formatted_path,
            Err(e) => {
                return Err(e);
            }
        };

        // apply middlewares
        let mut context = context::Context::new(request);
        for middleware in &self.middlewares {
            context = (middleware)(context);
        }

        // request path pattern matching with registered route paths
        match self.routes.get(&context.request.path) {
            Some(path_map) => match path_map.get(&context.request.method.to_string()) {
                Some(route_handler) => {
                    // the request path, method `exactly` matches a registered route path, method
                    return Ok((route_handler)(context));
                }
                None => {
                    // the request path `exactly` matches a registered route path but the method is
                    // different
                    return Ok(response::Response::new(
                        utils::HttpStatusCode::MethodNotAllowed,
                        format!("{}", utils::HttpStatusCode::MethodNotAllowed.code().0).to_string(),
                    ));
                }
            },
            // the request path does not `exactly` match a registered route path
            None => {
                for (route_path, method_map) in &self.routes {
                    match WebRouter::match_dynamic_route(
                        context.request.path.to_string(),
                        route_path.to_string(),
                    ) {
                        Some(params) => match method_map.get(&context.request.method.to_string()) {
                            Some(route_handler) => {
                                // process and validate query parameters from request path
                                let mut query_params = HashMap::new();
                                match context.request.path.split('?').nth(1) {
                                    Some(query) => {
                                        for part in query.split('&') {
                                            let mut key_value = part.split('=');
                                            let key = key_value.next().unwrap_or("");
                                            let value = key_value.next().unwrap_or("");
                                            if key.is_empty() {
                                                // If the key is empty, return a bad request response
                                                return Ok(response::Response::new(
                                                    utils::HttpStatusCode::BadRequest,
                                                    format!(
                                                        "{}",
                                                        utils::HttpStatusCode::BadRequest.code().0
                                                    )
                                                    .to_string(),
                                                ));
                                            }
                                            query_params.insert(key.to_string(), value.to_string());
                                        }
                                    }
                                    None => {}
                                }

                                context.params = params;
                                context.query_params = query_params;

                                // the request path matches a registered dynamic route path pattern
                                // with provided parameters
                                return Ok((route_handler)(context));
                            }
                            None => {}
                        },
                        None => {}
                    }
                }
                // the request path neither `exactly` matches any registered route,
                // nor matches with any registered dynamic route path pattern
                return Ok(response::Response::new(
                    utils::HttpStatusCode::NotFound,
                    format!("{}", utils::HttpStatusCode::NotFound.code().0).to_string(),
                ));
            }
        }
    }
    // This function first removes the query parameters from the request path string, then
    // splits both the request path and route path into vectors by splitting at `/` (slashes).
    // It ensures the lengths of these vectors are the same. If they are, it zips the vectors
    // into one vector with the format `(request_path_part, route_path_part)`.
    //
    // It then loops over this vector and checks if the `route_path_part` of any item starts with `:`.
    // If it does, this registered route is identified as a dynamic route, so the corresponding
    // `request_path_part` is stored in the `params` `HashMap` which is then returned after the loop ends.
    // If the `route_path_part` does not start with `:`, it is treated as a normal route and both parts
    // must be equal. If they aren't, the function returns `None`.
    fn match_dynamic_route(
        request_path: String,
        route_path: String,
    ) -> Option<HashMap<String, String>> {
        let mut params: HashMap<String, String> = HashMap::new();

        let request_path_parts: Vec<&str> = request_path.split('?').collect::<Vec<_>>()[0]
            .split('/')
            .collect();
        let route_path_parts: Vec<&str> = route_path.split('/').collect();

        if route_path_parts.len() != request_path_parts.len() {
            return None;
        }

        for (request_path_part, route_path_part) in
            request_path_parts.iter().zip(route_path_parts.iter())
        {
            if route_path_part.starts_with(':') {
                let param_name = &route_path_part[1..];
                params.insert(param_name.to_string(), request_path_part.to_string());
            } else if request_path_part != route_path_part {
                return None;
            }
        }
        Some(params)
    }
}
