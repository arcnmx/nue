#![deny(missing_docs)]

//! Utilities for working with I/O streams.

extern crate byteorder;
extern crate uninitialized;
extern crate resize_slice;

/// An extension for `Read`ing an exact amount of data.
pub mod read_exact;

mod seek_forward;

mod buf_seeker;
mod region;
mod align;
mod take;

pub use seek_forward::{
    SeekRewind, SeekForward, SeekBackward, SeekAbsolute, SeekEnd, Tell,
    ReadWriteTell, SeekForwardRead, SeekForwardWrite, SeekAbsoluteRewind, SeekAll
};
pub use read_exact::ReadExactExt;
pub use buf_seeker::BufSeeker;
pub use region::Region;
pub use align::SeekAlignExt;
pub use take::Take;
