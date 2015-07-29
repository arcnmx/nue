use std::io::{self, Read, Write};
use std::cmp::min;
use seek_forward::{SeekForward, Tell};

/// Wraps around a stream to limit the length of the underlying stream.
///
/// This implementation differs from `std::io::Take` in that it also allows writes,
/// and seeking forward is allowed if the underlying stream supports it.
pub struct Take<T> {
    inner: T,
    limit: u64,
}

impl<T> Take<T> {
    /// Creates a new `Take` with `limit` bytes
    pub fn new(inner: T, limit: u64) -> Self {
        Take {
            inner: inner,
            limit: limit,
        }
    }
}

impl<T: Write> Write for Take<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let limit = min(self.limit, buf.len() as u64);

        if limit == 0 {
            return Ok(0)
        }

        let buf = &buf[..limit as usize];
        let inner = try!(self.inner.write(buf));
        self.limit -= inner as u64;
        Ok(inner)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Read> Read for Take<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let limit = min(self.limit, buf.len() as u64);

        if limit == 0 {
            return Ok(0)
        }

        let buf = &mut buf[..limit as usize];
        let inner = try!(self.inner.read(buf));
        self.limit -= inner as u64;
        Ok(inner)
    }
}

impl<T: SeekForward> SeekForward for Take<T> {
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        let res = try!(self.inner.seek_forward(min(offset, self.limit)));
        self.limit -= res;
        Ok(res)
    }
}

impl<T: Tell> Tell for Take<T> {
    fn tell(&mut self) -> io::Result<u64> {
        self.inner.tell()
    }
}
