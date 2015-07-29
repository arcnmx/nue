#![feature(plugin_registrar, rustc_private)]

//! `#[derive(..)]` attributes for POD and binary encodable types.
//!
//! # Attributes
//!
//! ## `#[packed]`
//!
//! Applies `#[repr(Packed)]` while also ensuring that all members safely allow
//! unaligned access.
//!
//! ```
//! #![feature(plugin, custom_derive, custom_attribute)]
//! #![plugin(nue_macros)]
//!
//! extern crate nue;
//! use nue::{Pod, Aligned, Un};
//!
//! # fn main() {
//! #[packed]
//! struct Data(u8, Un<u32>);
//!
//! let data = Data(5, 5.unaligned());
//! assert_eq!(data.0, 5u8);
//! assert_eq!(u32::aligned(data.1), 5u32);
//! # }
//! ```
//!
//! ## `#[derive(Pod)]`
//!
//! Marks a struct as `pod::Pod`. It must only contain other `Pod` members, and
//! the type must be packed.
//!
//! ### `#[derive(PodPacked)]`
//!
//! Marks a struct as `pod::Pod`, and also applies the `#[packed]`
//! attribute to the type.
//!
//! ### Example
//!
//! ```
//! #![feature(plugin, custom_derive, custom_attribute)]
//! #![plugin(nue_macros)]
//!
//! extern crate pod;
//! extern crate nue;
//! use pod::PodExt;
//!
//! # fn main() {
//! #[derive(Pod)]
//! #[packed]
//! struct Data(u8);
//!
//! assert_eq!(Data(5).as_slice(), &[5]);
//! # }
//! ```
//!
//! ## `#[derive(NueEncode, NueDecode)]`
//!
//! Implements `nue::Encode` and `nue::Decode` on the struct.
//! All fields must also implement `Encode` / `Decode` (or be skipped by a `nue` attribute).
//!
//! ### `#[nue(...)]`, `#[nue_enc(...)]`, `#[nue_dec(...)]`
//!
//! Additional coding options may be provided per field using the `nue` attributes.
//! They will affect how the parent type is encoded or decoded. The attributes accept
//! arbitrary Rust expressions. Member variables may be accessed through `self`.
//! 
//! `nue_enc` only applies to encoding, `nue_dec` applies to decoding, and `nue` applies to both.
//! The order of the attributes doesn't usually matter, though `align` and `skip` interact
//! differently depending on which is defined first.
//!
//! #### `assert`
//!
//! Asserts that some property is true before continuing with the operation.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Decode;
//!
//! # fn main() {
//! #[derive(NueDecode)]
//! struct Data(
//!     #[nue(assert = "self.0 == 0")]
//! 	u8
//! );
//!
//! let data = &[1];
//! assert!(&Data::decode_slice(data).is_err());
//!
//! let data = &[0];
//! assert!(&Data::decode_slice(data).is_ok());
//! # }
//! ```
//!
//! #### `align`
//!
//! Aligns the field to an offset of the given multiple.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data(
//! 	u8,
//! 	#[nue(align = "self.0 as u64 + 1")]
//! 	&'static str
//! );
//!
//! let data = Data(2, "hi");
//! let cmp = &[2, 0, 0, b'h', b'i'];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `skip`
//!
//! Discards the provided amount of bytes before encoding/decoding the value.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data(
//! 	u8,
//! 	#[nue(skip = "1")]
//! 	&'static str
//! );
//!
//! let data = Data(2, "hi");
//! let cmp = &[2, 0, b'h', b'i'];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `cond`
//!
//! Conditionally encodes or decodes the field. If the condition is not met,
//! `Default::default()` will be used.
//! `false` is a static guarantee that the field will be ignored.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data<'a>(
//! 	u8,
//! 	#[nue(cond = "false")]
//! 	&'a () // Note that this type does not implement `Encode`
//! );
//!
//! let u = ();
//! let data = Data(2, &u);
//! let cmp = &[2];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `default`
//!
//! Determines the default value to be used if `cond` evaluates to false.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Decode;
//!
//! # fn main() {
//! #[derive(NueDecode, PartialEq, Debug)]
//! struct Data(
//! 	u8,
//! 	#[nue(cond = "self.0 == 1", default = "5")]
//! 	u8
//! );
//!
//! let data = &[2];
//! assert_eq!(&Data::decode_slice(data).unwrap(), &Data(2, 5));
//!
//! let data = &[1, 2];
//! assert_eq!(&Data::decode_slice(data).unwrap(), &Data(1, 2));
//! # }
//! ```
//!
//! #### `limit`
//!
//! Limits the amount of bytes that can be consumed or written during coding.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Decode;
//!
//! # fn main() {
//! #[derive(NueDecode)]
//! struct Data(
//! 	#[nue(limit = "4")]
//! 	String,
//! );
//!
//! let data = b"hello";
//! assert_eq!(&Data::decode_slice(data).unwrap().0, "hell");
//! # }
//! ```
//!
//! #### `consume`
//!
//! When set, uses all of `limit` even if the type did not encode or decode the entire byte region.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate nue;
//! use nue::Decode;
//! use std::ffi::CString;
//!
//! # fn main() {
//! #[derive(NueDecode)]
//! struct Data(
//! 	#[nue(limit = "8", consume = "true")]
//! 	CString,
//! 	u8
//! );
//!
//! let data = b"hello\0\0\0\x05";
//! let data = Data::decode_slice(data).unwrap();
//! assert_eq!(data.0.to_bytes(), b"hello");
//! assert_eq!(data.1, 5);
//! # }
//! ```

extern crate rustc;
extern crate nue_codegen;

#[plugin_registrar]
#[doc(hidden)]
pub fn register(reg: &mut rustc::plugin::Registry) {
    nue_codegen::register(reg);
}
