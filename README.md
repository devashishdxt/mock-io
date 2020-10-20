# mock-io

[![CI](https://github.com/devashishdxt/mock-io/workflows/CI/badge.svg)](https://github.com/devashishdxt/mock-io/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/mock-io)](https://crates.io/crates/mock-io)
[![Documentation](https://docs.rs/mock-io/badge.svg)](https://docs.rs/mock-io)
[![License](https://img.shields.io/crates/l/mock-io)](https://github.com/devashishdxt/mock-io/blob/master/LICENSE-MIT)

A crate with mock IO stream and listener implementations.

## Usage

Add `mock-io` in your `Cargo.toml`'s `dependencies` section:

```toml
[dependencies]
mock-io = "0.1"
```

Here is a sample usage of this crate:

```rust
use mock_io::sync::{MockListener, MockStream};

let (listener, handle) = MockListener::new();

thread::spawn(move || {
    let mut stream = MockStream::connect(&handle).unwrap();
    stream.write(&1u64.to_be_bytes()).unwrap();
    stream.write(&2u64.to_be_bytes()).unwrap();
});

while let Ok(mut stream) = listener.accept() {
    let mut buf = [0; 8];

    stream.read(&mut buf).unwrap();
    assert_eq!(1u64.to_be_bytes(), buf);
    
    stream.read(&mut buf).unwrap();
    assert_eq!(2u64.to_be_bytes(), buf);
}

```

### Features

- `sync`: Enables sync mock IO stream and listener
  - **Enabled** by default
- `async-futures`: Enables async mock IO stream and listener (using `futures::io::{AsyncRead, AsyncWrite}`)
  - **Disabled** by default
- `async-tokio`: Enables async mock IO stream and listener (using `tokio::io::{AsyncRead, AsyncWrite}`)
  - **Disabled** by default

> Note: Some functions in this crate returns a `Future`. So, you'll need an executor to drive `Future`s returned
from these functions. `async-std` and `tokio` are two popular options.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
