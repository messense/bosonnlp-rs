//! [`BosonNLP`](http://bosonnlp.com) SDK for Rust
//!
//!
//! ## 安装
//!
//! 在 ``Cargo.toml`` 增加如下内容:
//!
//! ```toml
//! [dependencies]
//! bosonnlp = "0.9"
//! ```
//!
//! ## 使用教程
//!
//! API Token 申请请访问 http://bosonnlp.com
//!
//! ```
//! extern crate bosonnlp;
//!
//! use bosonnlp::BosonNLP;
//!
//! fn main() {
//!     let nlp = BosonNLP::new(env!("BOSON_API_TOKEN"));
//!     let rs = nlp.sentiment(&["这家味道还不错"], "food").unwrap();
//!     assert_eq!(1, rs.len());
//! }
//! ```
//!
//! 可以在 [`BosonNLP` 文档网站](http://docs.bosonnlp.com) 阅读详细的 `BosonNLP` REST API 文档。
#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
extern crate url;
extern crate uuid;
extern crate reqwest;
extern crate flate2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate failure;
#[macro_use]
extern crate failure_derive;

mod rep;
mod client;
mod task;
mod errors;

pub use self::client::BosonNLP;
pub use self::errors::*;
pub use self::rep::*;
