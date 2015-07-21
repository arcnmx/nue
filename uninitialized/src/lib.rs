#![deny(missing_docs)]

//! Provides the option to use uninitialized memory for performance improvements.
//!
//! `uninitialized()` is backed by `std::mem::zeroed()` unless the feature is toggled on.
//! Downstream binary crates that want to take advantage of `std::mem::uninitialized()`
//! should use the following in `Cargo.toml`:
//!
//! ```toml
//! [dependencies.uninitialized]
//! version = "*"
//! features = ["uninitialized"]
//! ```

#[cfg(feature = "uninitialized")]
pub use std::mem::uninitialized as uninitialized;

#[cfg(not(feature = "uninitialized"))]
pub use std::mem::zeroed as uninitialized;

/// A constant indicating whether the `uninitialized` feature is enabled.
#[cfg(feature = "uninitialized")]
pub const UNINITIALIZED: bool = true;

/// A constant indicating whether the `uninitialized` feature is enabled.
#[cfg(not(feature = "uninitialized"))]
pub const UNINITIALIZED: bool = false;
