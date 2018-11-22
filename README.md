# bosonnlp-rs

[![Build Status](https://travis-ci.org/messense/bosonnlp-rs.svg?branch=master)](https://travis-ci.org/messense/bosonnlp-rs)
[![Coverage Status](https://coveralls.io/repos/messense/bosonnlp-rs/badge.svg)](https://coveralls.io/r/messense/bosonnlp-rs)
[![Crates.io](https://img.shields.io/crates/v/bosonnlp.svg)](https://crates.io/crates/bosonnlp)
[![docs.rs](https://docs.rs/bosonnlp/badge.svg)](https://docs.rs/bosonnlp/)

[BosonNLP](http://bosonnlp.com) SDK for [Rust](http://rust-lang.org)

## Installation

Add it to your ``Cargo.toml``:

```toml
[dependencies]
bosonnlp = "0.9"
```

Add ``extern crate bosonnlp`` to your crate root and your're good to go!

## Build

```bash
$ cargo build --release
```

## Test

First you need to export a shell variable called ``BOSON_API_TOKEN`` and then run:

```bash
$ cargo test
```

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
