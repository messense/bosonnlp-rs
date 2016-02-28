use std::fmt;
use std::error;
use std::io;

use hyper::Error as HTTPError;
use hyper::status::StatusCode;
use rustc_serialize::json::{DecoderError, EncoderError};


#[derive(Debug)]
pub enum Error {
    Encoding(EncoderError),
    Decoding(DecoderError),
    Io(io::Error),
    Http(HTTPError),
    Api {
        code: StatusCode,
        reason: String,
    },
    TaskNotFound { task_id: String },
    Timeout { task_id: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Encoding(ref err) => err.fmt(f),
            Error::Decoding(ref err) => err.fmt(f),
            Error::Io(ref err) => err.fmt(f),
            Error::Http(ref err) => err.fmt(f),
            Error::Api { code, ref reason } => write!(f, "API error, code {}, reason {}", code, reason),
            Error::TaskNotFound { ref task_id } => write!(f, "cluster {} not found", task_id),
            Error::Timeout { ref task_id } => write!(f, "cluster {} timed out", task_id),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Encoding(ref err) => err.description(),
            Error::Decoding(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::Http(ref err) => err.description(),
            Error::Api { ref reason, .. } => reason,
            Error::TaskNotFound { .. } => "cluster task not found",
            Error::Timeout { .. } => "cluster task timed out",
        }
    }
}

impl From<DecoderError> for Error {
    fn from(error: DecoderError) -> Error {
        Error::Decoding(error)
    }
}

impl From<EncoderError> for Error {
    fn from(error: EncoderError) -> Error {
        Error::Encoding(error)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<HTTPError> for Error {
    fn from(err: HTTPError) -> Error {
        Error::Http(err)
    }
}
