#![cfg_attr(feature = "unstable", feature(optin_builtin_traits, raw, box_raw))]
#![deny(missing_docs)]

//! Provides traits that assist with I/O and byte slice conversions involving Plain Old Data.
//!
//! # Nightly / Unstable
//!
//! The `unstable` cargo feature can be used for safe automagic derives thanks to OIBIT
//!
//! # Example
//!
//! ```
//! use pod::{Pod, PodExt, Le, Be};
//!
//! impl Pod for Data { }
//! # #[cfg(not(feature = "unstable"))]
//! # unsafe impl pod::PodType for Data { }
//!
//! #[repr(packed)]
//! struct Data(u8, Le<u16>, Be<u32>);
//!
//! let data = Data(1, Le::new(0x2055), Be::new(0xdeadbeef));
//!
//! let cmp = &[
//!     0x01,
//!     0x55, 0x20,
//!     0xde, 0xad, 0xbe, 0xef,
//! ];
//!
//! assert_eq!(cmp, data.as_slice());
//!
//! ```

extern crate uninitialized;
extern crate byteorder;
extern crate nue_io;

mod pod;

/// I/O traits for POD and other types.
pub mod code;

/// Containers for primitives
pub mod endian;

pub use endian::{Le, Be};
pub use code::{Encode, Decode};
pub use pod::{Pod, PodType, PodExt};
