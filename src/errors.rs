use std::io;

use reqwest::{self, StatusCode};
use serde_json;

#[derive(Debug, Fail)]
pub enum Error {
    /// API 错误
    #[fail(display = "API error, code {}, reason {}", code, reason)]
    Api {
        code: StatusCode,
        reason: String
    },

    /// 聚类任务未找到
    #[fail(display = "Cluster task {} not found", _0)]
    TaskNotFound(String),

    /// 聚类任务超时
    #[fail(display = "Cluster task {} timed out", _0)]
    Timeout(String),

    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "Http error: {}", _0)]
    Http(#[cause] reqwest::Error),

    #[fail(display = "Json error: {}", _0)]
    Json(#[cause] serde_json::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}
