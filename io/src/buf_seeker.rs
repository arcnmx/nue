use std::io::{self, Read, BufRead};
use std::cmp::min;
use resize_slice::SliceExt;
use seek_forward::{SeekForward, SeekBackward, SeekRewind, SeekAbsolute, SeekEnd, Tell};

const DEFAULT_BUF_SIZE: usize = 0x400 * 0x40;

/// A buffered reader that allows for seeking within the buffer.
///
/// Unlike `std::io::BufReader`, seeking doesn't invalidate the buffer.
pub struct BufSeeker<T> {
    inner: T,
    buf: Vec<u8>,
    pos: usize,
}

impl<T> BufSeeker<T> {
    /// Creates a new `BufSeeker` around the specified `Read`.
    pub fn new(inner: T) -> Self {
        BufSeeker::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a `BufSeeker` with a specific buffer size.
    pub fn with_capacity(cap: usize, inner: T) -> Self {
        BufSeeker {
            inner: inner,
            buf: Vec::with_capacity(cap),
            pos: 0,
        }
    }

    /// Unwraps the `BufSeeker`, returning the underlying reader.
    ///
    /// Note that any leftover data in the buffer will be lost.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: SeekForward> SeekForward for BufSeeker<T> {
    #[inline]
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        let pos = (self.buf.len() - self.pos) as u64;
        if offset <= pos as u64 {
            self.pos += offset as usize;
            Ok(offset)
        } else {
            let offset = offset - pos;
            let res = try!(self.inner.seek_forward(offset));
            self.pos = self.buf.len();
            Ok(res + pos)
        }
    }
}

impl<T: SeekBackward> SeekBackward for BufSeeker<T> {
    #[inline]
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        let pos = self.pos as u64;
        if offset <= pos {
            self.pos -= offset as usize;
            Ok(offset)
        } else {
            let offset = offset + pos;
            let res = try!(self.inner.seek_backward(offset));
            self.pos = self.buf.len();
            Ok(res - pos)
        }
    }
}

impl<T: SeekRewind> SeekRewind for BufSeeker<T> {
    #[inline]
    fn seek_rewind(&mut self) -> io::Result<()> {
        try!(self.inner.seek_rewind());
        self.pos = self.buf.len();
        Ok(())
    }
}

impl<T: Tell> Tell for BufSeeker<T> {
    #[inline]
    fn tell(&mut self) -> io::Result<u64> {
        self.inner.tell().map(|v| v - (self.buf.len() - self.pos) as u64)
    }
}

impl<T: SeekAbsolute> SeekAbsolute for BufSeeker<T> {
    #[inline]
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        let pos = try!(self.inner.seek_absolute(pos));
        self.pos = self.buf.len();
        Ok(pos)
    }
}

impl<T: SeekEnd> SeekEnd for BufSeeker<T> {
    #[inline]
    fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
        let pos = try!(self.inner.seek_end(offset));
        self.pos = self.buf.len();
        Ok(pos)
    }
}

impl<T: Read> Read for BufSeeker<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.buf.len() && buf.len() >= self.buf.capacity() {
            self.inner.read(buf)
        } else {
            let read = buf.copy_from(try!(self.fill_buf()));
            self.consume(read);
            Ok(read)
        }
    }
}

impl<T: Read> BufRead for BufSeeker<T> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        use std::slice::from_raw_parts_mut;

        if self.pos >= self.buf.len() {
            unsafe {
                let buf = from_raw_parts_mut(self.buf.as_mut_ptr(), self.buf.capacity());
                self.buf.set_len(try!(self.inner.read(buf)));
                self.pos = 0;
            }
        }
        Ok(&self.buf[self.pos..])
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.pos = min(self.pos + amt, self.buf.len());
    }
}
