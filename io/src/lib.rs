#![deny(missing_docs)]

//! Utilities for working with I/O streams.

extern crate byteorder;
extern crate resize_slice;

/// An extension for `Read`ing an exact amount of data.
pub mod read_exact;

pub use read_exact::ReadExact;
