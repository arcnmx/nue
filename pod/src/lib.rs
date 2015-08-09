#![cfg_attr(feature = "unstable", feature(box_raw))]
#![deny(missing_docs)]

//! Provides traits that assist with I/O and byte slice conversions involving Plain Old Data.
//!
//! # Safety
//!
//! The `nue-macros` crate can be used for safe automagic derives.
//!
//! # Example
//!
//! ```
//! use pod::{Pod, Le, Be};
//! # #[cfg(not(feature = "unstable"))]
//! # mod stable {
//! # use pod::packed::{Unaligned, Packed};
//! # unsafe impl Packed for super::Data { }
//! # unsafe impl Unaligned for super::Data { }
//! # }
//!
//! unsafe impl Pod for Data { }
//!
//! #[repr(C)]
//! struct Data(u8, Le<u16>, Be<u32>);
//!
//! # fn main() {
//! let data = Data(1, Le::new(0x2055), Be::new(0xdeadbeef));
//!
//! let cmp = &[
//!     0x01,
//!     0x55, 0x20,
//!     0xde, 0xad, 0xbe, 0xef,
//! ];
//!
//! assert_eq!(cmp, data.as_slice());
//! # }
//!
//! ```

extern crate uninitialized;
extern crate byteorder;
extern crate packed as nue_packed;
extern crate nue_io;

mod pod;

/// I/O traits for POD and other types.
pub mod code;

/// Containers for primitives
pub mod endian;

pub use endian::{Le, Be, Native};
pub use code::{Encode, Decode};
pub use pod::Pod;

/// Re-export the `packed` crate
pub use nue_packed as packed;
