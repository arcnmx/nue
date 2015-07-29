use std::io::{self, Read};

pub use byteorder::Error;

/// Extension trait that provides `read_exact` for all `Read` implementations.
pub trait ReadExactExt {
    /// Reads into the entirety of `buf` or fails with an error.
    ///
    /// The `Read` equivalent of `Write::write_all`.
    /// Retries upon `Interrupted` errors.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    /// Reads as much as possible into `buf` until EOF.
    ///
    /// Retries upon `Interrupted` errors.
    fn read_exact_eof(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

impl<R: Read> ReadExactExt for R {
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        match self.read_exact_eof(buf) {
            Ok(len) if len == buf.len() => Ok(()),
            Ok(_) => Err(Error::UnexpectedEOF),
            Err(e) => Err(e.into()),
        }
    }

    #[inline]
    fn read_exact_eof(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;
        while n < buf.len() {
            match self.read(&mut buf[n..]) {
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e),
                Ok(0) => break,
                Ok(len) => n += len,
            }
        }

        Ok(n)
    }
}
