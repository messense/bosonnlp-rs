//! [BonsonNLP](http://bosonnlp.com) SDK for Rust
//!
//!
//! ## 安装
//!
//! 在 ``Cargo.toml`` 增加如下内容:
//!
//! ```toml
//! [dependencies]
//! bosonnlp = "0.2"
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
//!     let rs = nlp.sentiment(&vec!["这家味道还不错".to_owned()], "food").unwrap();
//!     assert_eq!(1, rs.len());
//! }
//! ```
//!
//! 可以在 [BosonNLP 文档网站](http://docs.bosonnlp.com) 阅读详细的 BosonNLP REST API 文档。
#![recursion_limit = "1024"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(cyclomatic_complexity))]
#![cfg_attr(feature="clippy", allow(used_underscore_binding))]
#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#[macro_use]
extern crate log;
extern crate url;
extern crate uuid;
#[macro_use]
extern crate hyper;
extern crate flate2;
extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

mod rep;
mod client;
pub mod task;
mod errors;

pub use self::client::BosonNLP;
pub use self::errors::{ErrorKind, Result};
pub use self::rep::*;
