# bosonnlp-rs

[BosonNLP](http://bosonnlp.com) SDK for [Rust](http://rust-lang.org)

[![Build Status](https://travis-ci.org/messense/bosonnlp-rs.svg?branch=master)](https://travis-ci.org/messense/bosonnlp-rs)
[![Coverage Status](https://coveralls.io/repos/messense/bosonnlp-rs/badge.svg)](https://coveralls.io/r/messense/bosonnlp-rs)
[![Crates.io](https://img.shields.io/crates/v/bosonnlp.svg)](https://crates.io/crates/bosonnlp)


## Installation

Add it to your ``Cargo.toml``:

```toml
[dependencies]
bosonnlp = "0.2"
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

## [Documentation](http://messense.github.io/bosonnlp-rs/bosonnlp/index.html)

For BosonNLP REST API documentation, please visit http://docs.bosonnlp.com/index.html

To generate crate documentation locally, run:

```bash
$ cargo doc
```

Then you will be able to open the documentation under ``target/doc/bosonnlp`` directory.

Wait, of course you can read it [online](http://messense.github.io/bosonnlp-rs/bosonnlp/index.html).

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
