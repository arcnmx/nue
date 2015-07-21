use std::io::{self, Read, Write, BufRead, Seek, SeekFrom};
use std::cmp::{min, max};

/// Creates an isolated segment of an underlying stream.
///
/// Seeks past the region will be capped, and reaching the end of
/// the region will result in EOF when reading or writing.
pub struct Region<T> {
    inner: T,
    pos: Option<u64>,
    start: u64,
    end: u64,
}

impl<T> Region<T> {
    /// Creates a new `Region` at the specified offsets of `inner`.
    pub fn new(start: u64, end: u64, inner: T) -> Self {
        Region {
            inner: inner,
            pos: None,
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

impl<T: Seek> Region<T> {
    fn position(&mut self) -> io::Result<u64> {
        let pos = match self.pos {
            Some(pos) => pos,
            None => try!(self.inner.seek(SeekFrom::Current(0))),
        };

        let pos = if pos < self.start {
            try!(self.inner.seek(SeekFrom::Start(self.start)))
        } else {
            pos
        };

        self.pos = Some(pos);

        Ok(pos - self.start)
    }
}

fn limit(end: u64, pos: u64, len: usize) -> usize {
    let limit = if pos >= end {
        0
    } else {
        end - pos
    };

    if limit < len as u64 {
        limit as usize
    } else {
        len
    }
}

impl<T: Read + Seek> Read for Region<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let pos = try!(self.position());
        let len = limit(self.end, pos, buf.len());

        if len == 0 {
            Ok(0)
        } else {
            let read = try!(self.inner.read(&mut buf[..len]));
            self.pos = Some(pos.saturating_add(read as u64));
            Ok(read)
        }
    }
}

impl<T: Write + Seek> Write for Region<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pos = try!(self.position());
        let len = limit(self.end, pos, buf.len());

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

impl<T: Seek> Seek for Region<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(pos) => SeekFrom::Start(min(self.start.saturating_add(pos), self.end)),
            SeekFrom::End(pos) => SeekFrom::Start(if pos < 0 {
                self.end.saturating_sub(-pos as u64)
            } else {
                self.end.saturating_add(pos as u64)
            }),
            SeekFrom::Current(pos) => SeekFrom::Current(pos),
        };
        let pos = try!(self.inner.seek(pos));
        self.pos = Some(pos);
        Ok(min(max(self.start, pos), self.end) - self.start)
    }
}

impl<T: BufRead + Seek> BufRead for Region<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let pos = try!(self.position());

        let buf = try!(self.inner.fill_buf());
        let len = limit(self.end, pos, buf.len());

        Ok(&buf[0..len])
    }

    fn consume(&mut self, amt: usize) {
        if let Some(pos) = self.pos {
            let amt = limit(self.end, pos, amt);
            self.inner.consume(amt);
            self.pos = Some(pos.saturating_add(amt as u64));
        }
    }
}
