#[cfg(feature = "serde_derive")]
include!("rep.in.rs");

#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/rep.rs"));
