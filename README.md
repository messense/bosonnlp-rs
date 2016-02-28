# bosonnlp-rs

[BosonNLP](http://bosonnlp.com) SDK for [Rust](http://rust-lang.org)

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

The MIT License (MIT)

Copyright (c) 2016 BosonData 

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
