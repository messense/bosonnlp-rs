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
#[macro_use]
extern crate log;
extern crate url;
extern crate uuid;
#[macro_use]
extern crate hyper;
extern crate jsonway;
extern crate rustc_serialize;
extern crate flate2;

mod rep;
mod client;
pub mod task;
mod errors;

pub use self::client::{BosonNLP, Result};
pub use self::errors::Error;
pub use self::rep::*;
