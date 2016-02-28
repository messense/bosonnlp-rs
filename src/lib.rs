//! [BonsonNLP](http://bosonnlp.com) SDK for Rust
#[macro_use] extern crate log;
extern crate url;
extern crate uuid;
#[macro_use] extern crate hyper;
extern crate jsonway;
extern crate rustc_serialize;
extern crate flate2;

mod rep;
mod client;
mod task;
mod errors;

pub use self::client::{BosonNLP, Result};
pub use self::errors::Error;
pub use self::rep::*;
