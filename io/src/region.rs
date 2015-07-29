use std::io::{self, Read, Write, BufRead};
use std::cmp::min;
use seek_forward::{SeekForward, SeekAbsolute, Tell, SeekRewind, SeekEnd, SeekBackward};

/// Creates an isolated segment of an underlying stream.
///
/// Seeks past the region will be capped, and reaching the end of
/// the region will result in EOF when reading or writing.
pub struct Region<T> {
    inner: T,
    start: u64,
    end: u64,
}

impl<T> Region<T> {
    /// Creates a new `Region` at the specified offsets of `inner`.
    pub fn new(inner: T, start: u64, end: u64) -> Self {
        Region {
            inner: inner,
            start: start,
            end: end,
        }
    }

    /// Returns the region bounds.
    pub fn region(&self) -> (u64, u64) {
        (self.start, self.end)
    }

    /// Unwraps the `Region` to return the inner stream.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Tell + SeekAbsolute> Region<T> {
    fn position(&mut self) -> io::Result<u64> {
        let pos = try!(self.inner.tell());

        if pos < self.start {
            self.inner.seek_absolute(self.start)
        } else {
            Ok(pos)
        }
    }

    fn limit(&mut self, len: u64) -> io::Result<u64> {
        let pos = try!(self.position());
        let end = self.end;

        Ok(limit(pos, end, len))
    }
}

fn limit(pos: u64, end: u64, len: u64) -> u64 {
    let limit = if pos >= end {
        0
    } else {
        end - pos
    };

    min(limit, len)
}

impl<T: Read + Tell + SeekAbsolute> Read for Region<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = try!(self.limit(buf.len() as u64)) as usize;

        if len == 0 {
            Ok(0)
        } else {
            let read = try!(self.inner.read(&mut buf[..len]));
            Ok(read)
        }
    }
}

impl<T: Write + Tell + SeekAbsolute> Write for Region<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = try!(self.limit(buf.len() as u64)) as usize;

        if len == 0 {
            Ok(0)
        } else {
            self.inner.write(&buf[..len])
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: SeekAbsolute> SeekAbsolute for Region<T> {
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        self.inner.seek_absolute(min(self.start.saturating_add(pos), self.end)).map(|v| v - self.start)
    }
}

impl<T: SeekForward + Tell + SeekAbsolute> SeekForward for Region<T> {
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        let offset = try!(self.limit(offset));
        self.inner.seek_forward(offset)
    }
}

impl<T: Tell + SeekBackward> SeekBackward for Region<T> {
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        let off = try!(self.inner.tell()).saturating_sub(self.start);;
        let offset = min(off, offset);
        self.inner.seek_backward(offset)
    }
}

impl<T: SeekAbsolute> SeekRewind for Region<T> {
    fn seek_rewind(&mut self) -> io::Result<()> {
        self.inner.seek_absolute(self.start).map(|_| ())
    }
}

impl<T: SeekAbsolute> SeekEnd for Region<T> {
    fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
        self.inner.seek_absolute((self.end as i64 + offset) as u64).map(|v| v - self.start)
    }
}

impl<T: Tell> Tell for Region<T> {
    fn tell(&mut self) -> io::Result<u64> {
        self.inner.tell().map(|v| min(self.end, v.saturating_sub(self.start)))
    }
}

impl<T: BufRead + Tell + SeekAbsolute> BufRead for Region<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let pos = try!(self.position());

        let buf = try!(self.inner.fill_buf());
        let len = limit(pos, self.end, buf.len() as u64) as usize;

        Ok(&buf[..len])
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use super::Region;
    use read_exact::ReadExactExt;
    use seek_forward::{SeekAll, SeekAbsolute};

    fn data(count: usize) -> Vec<u8> {
        use std::iter::{repeat};
        repeat(0).enumerate().map(|(i, _)| i as u8).take(count).collect()
    }

    #[test]
    fn region() {
        let data = data(0x100);
        let cursor = SeekAll::new(Cursor::new(data.clone()));

        let mut region = Region::new(cursor, 0x40, 0x80);
        let mut odata = vec![0u8; 0x40];
        region.read_exact(&mut odata).unwrap();
        assert_eq!(&odata[..], &data[0x40..0x80]);

        region.seek_absolute(0x20).unwrap();
        region.read_exact(&mut odata[..0x20]).unwrap();
        assert_eq!(&odata[..0x20], &data[0x60..0x80]);

        region.seek_absolute(0).unwrap();
        region.read_exact(&mut odata[..]).unwrap();
        assert_eq!(&odata[..], &data[0x40..0x80]);
    }
}
