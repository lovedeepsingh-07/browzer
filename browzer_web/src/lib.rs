use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::Arc,
};

pub mod context;
pub mod error;
pub mod request;
pub mod response;
pub mod router;
pub mod utils;


#[derive(Debug)]
pub struct WebServer {
    pub listener: TcpListener,
    request_pool: utils::thread_pool::ThreadPool,
    pub hide_banner: bool,
    pub address: String,
    router: Arc<router::WebRouter>,
}

impl WebServer {
    pub fn new(address: String, workers: usize) -> WebServer {
        let listener = match TcpListener::bind(&address) {
            Ok(listener) => listener,
            Err(listener_create_err) => {
                panic!(
                    "Failed to create listener for the WebServer, Error: {}",
                    listener_create_err.to_string()
                );
            }
        };

        let request_pool = utils::thread_pool::ThreadPool::new(workers);

        // return the WebServer struct
        return WebServer {
            listener,
            request_pool,
            hide_banner: false,
            address,
            router: Arc::new(router::WebRouter::new()),
        };
    }

    // This method allows you to register a new middleware function in the ruoter's middleware
    // vector, which applies all your registered middlewares to incoming requests one-by-one in
    // exact order in which you defined those middleware functions
    pub fn middleware<F>(&mut self, middleware_func: F)
    where
        F: Fn(context::Context) -> context::Context + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => router.add_middleware(Box::new(middleware_func)),
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => {
                match router.add(path.to_string(), utils::HttpMethod::GET, Box::new(handler)) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                    }
                }
            }
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => {
                match router.add(path.to_string(), utils::HttpMethod::POST, Box::new(handler)) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                    }
                }
            }
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => {
                match router.add(
                    path.to_string(),
                    utils::HttpMethod::PATCH,
                    Box::new(handler),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                    }
                }
            }
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }
    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(context::Context) -> response::Response + 'static + Send + Sync,
    {
        match Arc::get_mut(&mut self.router) {
            Some(router) => {
                match router.add(
                    path.to_string(),
                    utils::HttpMethod::DELETE,
                    Box::new(handler),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                    }
                }
            }
            None => eprintln!(
                "{}",
                error::WebServerError::InternalServerError(
                    "WebRouter is not innitialized".to_string()
                )
            ),
        };
    }

    // This method does it's function by registering a dynamic GET method route to the
    // `route_path`, that route's handler function gets the filename of the file that is requested
    // from the dynamic route params and then check if a file with that name exists under the
    // `dir_path`, if it does then the handler will return a `String` response with that file's
    // content as body, it not then it returns a `NotFound`
    pub fn serve_static(&mut self, dir_path: &str, route_path: &str) {
        let dir_path = Arc::new(dir_path.to_string());
        let dir_path_clone = Arc::clone(&dir_path);
        let route = format!("{}/:filename", route_path);

        self.get(&route, move |mut c| {
            let filename = match c.params.get("filename") {
                Some(filename) => filename,
                None => {
                    // Couldn't get the filename param
                    return c.send_string(
                        utils::HttpStatusCode::InternalServerError,
                        utils::HttpStatusCode::InternalServerError.code().0,
                    );
                }
            };
            let path = Path::new(&*dir_path_clone).join(filename); // NOTE: I have NO idea what is happening here
            match path.exists() {
                true => {
                    return c.send_string(
                        utils::HttpStatusCode::OK,
                        &match fs::read_to_string(path) {
                            Ok(res) => res,
                            Err(_) => {
                                // Couldn't prase the path to string
                                return c.send_string(
                                    utils::HttpStatusCode::InternalServerError,
                                    utils::HttpStatusCode::InternalServerError.code().0,
                                );
                            }
                        },
                    );
                }
                false => {
                    // filename doesn't exist under the dir_path
                    return c.send_string(
                        utils::HttpStatusCode::NotFound,
                        utils::HttpStatusCode::NotFound.code().0,
                    );
                }
            }
        });
    }

    // This method starts the web server, accepting incoming connections and distributing
    // them to worker threads for handling. It uses the `request_pool` to manage a pool of
    // worker threads and assigns incoming requests to these workers. The function will
    // continue to listen for connections indefinitely.
    pub fn listen(&self) {
        // print the server banner( a simple log message ) accoding to the `address` field boolean variable
        if !self.hide_banner {
            println!("-----> HTTP server running on {}", self.address);
        }

        // loop over incoming requests and send those request as jobs to the `request_pool` in
        // order to be distributed to the worker threads
        for stream in self.listener.incoming() {
            let router = Arc::clone(&self.router);
            match stream {
                Ok(stream) => {
                    match self.request_pool.execute(|| {
                        match Self::handle_request(router, stream) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Failed to handle incoming request, Error: {}", e);
                            }
                        };
                    }) {
                        Ok(_) => {}
                        Err(e) => eprintln!(
                            "Failed to assign Worker thread to incoming request, Error: {}",
                            e.to_string()
                        ),
                    };
                }
                Err(e) => {
                    eprintln!("Failed to establish a connection, Error: {}", e.to_string());
                }
            }
        }
    }

    fn handle_request(
        router: Arc<router::WebRouter>,
        mut stream: TcpStream,
    ) -> Result<(), error::WebServerError> {
        let mut buf_reader = BufReader::new(&mut stream);

        // parse the request string into a `Request` struct by first parsing the string to a string
        // vector containling the lines of requests as elements by following cases:-
        //
        // - if the headers contain the `Content-Length` header and it's value is more than 0, then
        //   we properly parse the body too
        // - if the headers do not contain the `Content-Length` then we stop after parsing
        //
        // and then passing that vector onto the `new` function of the `Request` string as input
        let request = match request::Request::new(&{
            let mut request_vector = Vec::new();
            let mut content_length = 0;

            for line in buf_reader.by_ref().lines() {
                let line = match line {
                    Ok(ln) => ln,
                    Err(e) => return Err(error::WebServerError::IO(e)),
                };
                match line.strip_prefix("Content-Length: ") {
                    Some(c_l) => {
                        content_length = match c_l.trim().parse() {
                            Ok(safe_c_l) => safe_c_l,
                            Err(e) => return Err(error::WebServerError::from(e)),
                        }
                    }
                    None => {}
                }
                if line.is_empty() {
                    request_vector.push(line);
                    break;
                }
                request_vector.push(line);
            }
            let mut body = Vec::new();
            if content_length > 0 {
                body.resize(content_length, 0);
                match buf_reader.take(content_length as u64).read_exact(&mut body) {
                    Ok(_) => {}
                    Err(e) => return Err(error::WebServerError::IO(e)),
                }
                request_vector.push(String::from_utf8_lossy(&body).to_string());
            }
            request_vector // return the request_vector to Request::new() function
        }) {
            Ok(safe) => safe,
            Err(e) => {
                return Err(error::WebServerError::RequestParseError(e));
            }
        };

        // utilize user registered routes from `routes` hashmap in the `WebRouter` to handle
        // requests, generate responses and then send those responses to the request agent throught
        // the TCP connection stream
        match stream.write_all(
            match router.handle_request(request) {
                Ok(res) => res.to_string(),
                Err(e) => {
                    return Err(error::WebServerError::InternalServerError(e.to_string()));
                }
            }
            .as_bytes(),
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(error::WebServerError::IO(e));
            }
        };

        match stream.flush() {
            Ok(_) => Ok({}),
            Err(e) => {
                return Err(error::WebServerError::StreamFlushError(e.to_string()));
            }
        }
    }
}
