use std::io::{self, Read};
use resize_slice::ResizeSlice;
use byteorder;

pub use byteorder::Error;

/// Extension trait that provides `read_exact` for all `Read` implementations.
pub trait ReadExact {
    /// Reads into the entirety of `buf` or fails with an error.
    ///
    /// The `Read` equivalent of `Write::write_all`.
    /// Retries upon `Interrupted` errors.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}

impl<R: Read> ReadExact for R {
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
        while buf.len() > 0 {
            match self.read(buf) {
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e.into()),
                Ok(0) => return Err(byteorder::Error::UnexpectedEOF),
                Ok(len) => buf.resize_from(len),
            }
        }

        Ok(())
    }
}
