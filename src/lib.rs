#![deny(missing_docs, unsafe_code)]
//! A crate with mock IO stream and listener implementations.
//!
//! ## Usage
//!
//! Add `mock-io` in your `Cargo.toml`'s `dependencies` section:
//!
//! ```toml
//! [dependencies]
//! mock-io = "0.3"
//! ```
//!
//! Here is a sample usage of this crate:
//!
//! ```rust
//! # use std::{thread, io::{Read, Write}};
//! use mock_io::sync::{MockListener, MockStream};
//!
//! let (listener, handle) = MockListener::new();
//!
//! thread::spawn(move || {
//!     let mut stream = MockStream::connect(&handle).unwrap();
//!     stream.write(&1u64.to_be_bytes()).unwrap();
//!     stream.write(&2u64.to_be_bytes()).unwrap();
//! });
//!
//! while let Ok(mut stream) = listener.accept() {
//!     let mut buf = [0; 8];
//!
//!     stream.read(&mut buf).unwrap();
//!     assert_eq!(1u64.to_be_bytes(), buf);
//!     
//!     stream.read(&mut buf).unwrap();
//!     assert_eq!(2u64.to_be_bytes(), buf);
//! }
//!
//! ```
//!
//! ### Features
//!
//! - `sync`: Enables sync mock IO stream and listener
//!   - **Enabled** by default
//! - `async-futures`: Enables async mock IO stream and listener (using `futures::io::{AsyncRead, AsyncWrite}`)
//!   - **Disabled** by default
//! - `async-tokio`: Enables async mock IO stream and listener (using `tokio::io::{AsyncRead, AsyncWrite}`)
//!   - **Disabled** by default
//!
//! > Note: Some functions in this crate returns a `Future`. So, you'll need an executor to drive `Future`s returned
//! from these functions. `async-std` and `tokio` are two popular options.
#![cfg_attr(feature = "doc", feature(doc_cfg))]

mod mock_listener;
mod mock_stream;

pub mod error;
#[cfg(feature = "async-futures")]
#[cfg_attr(feature = "doc", doc(cfg(feature = "async-futures")))]
pub mod futures;
#[cfg(feature = "sync")]
#[cfg_attr(feature = "doc", doc(cfg(feature = "sync")))]
pub mod sync;
#[cfg(feature = "async-tokio")]
#[cfg_attr(feature = "doc", doc(cfg(feature = "async-tokio")))]
pub mod tokio;
