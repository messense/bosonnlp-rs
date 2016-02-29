# bosonnlp-rs

[BosonNLP](http://bosonnlp.com) SDK for [Rust](http://rust-lang.org)


## Installation

Add it to your ``Cargo.toml``:

```toml
[dependencies]
bosonnlp = "0.1"
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

## Documentation

For BosonNLP REST API documentation, please visit http://docs.bosonnlp.com/index.html

To generate crate documentation locally, run:

```bash
$ cargo doc
```

Then you will be able to open the documentation under ``target/doc/bosonnlp`` directory.

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file.
