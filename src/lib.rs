//! [BonsonNLP](http://bosonnlp.com) SDK for Rust
#[macro_use] extern crate log;
extern crate url;
#[macro_use] extern crate hyper;
extern crate jsonway;
extern crate rustc_serialize;
extern crate flate2;

mod client;
mod errors;

pub use self::client::{BosonNLP, Result};
pub use self::errors::Error;
