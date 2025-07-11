use thiserror::Error;
use std::{
    io,
    num::ParseIntError,
    sync::{self, mpsc},
};

#[derive(Debug, Error)]
pub enum ThreadPoolError {
    #[error("Receiver lock error: {0}")]
    ReceiverLockError(String),

    #[error("Receive error: {0}")]
    ReceiveError(#[from] mpsc::RecvError),

    #[error("Send error: {0}")]
    SendError(String),
}

impl<T> From<sync::PoisonError<T>> for ThreadPoolError {
    fn from(err: sync::PoisonError<T>) -> Self {
        ThreadPoolError::ReceiverLockError(err.to_string())
    }
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Invalid request line: {0}")]
    InvalidRequestLineError(String),

    #[error("Empty HTTP request")]
    EmptyRequestError,
}

#[derive(Debug, Error)]
pub enum WebServerError {
    #[error("Stream flush error: {0}")]
    StreamFlushError(String),

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Request parse error: {0}")]
    RequestParseError(RequestError),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl From<ParseIntError> for WebServerError {
    fn from(err: ParseIntError) -> Self {
        WebServerError::IO(io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum WebRouterError {
    #[error("Error while formatting a path: {0}")]
    PathFormatError(String),
}
