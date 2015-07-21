use std::io::{self, Read, BufRead, BufReader, Seek, SeekFrom};

/// A buffered reader that allows for seeking within the buffer.
///
/// Unlike `std::io::BufReader`, seeking doesn't invalidate the buffer.
pub struct BufSeeker<T> {
    read: BufReader<T>,
    pos: Option<u64>,
}

impl<T: Read> BufSeeker<T> {
    /// Creates a new `BufSeeker` around the specified `Read`.
    pub fn new(inner: T) -> Self {
        BufSeeker {
            read: BufReader::new(inner),
            pos: None,
        }
    }

    /// Creates a `BufSeeker` with a specific buffer size.
    pub fn with_capacity(cap: usize, inner: T) -> Self {
        BufSeeker {
            read: BufReader::with_capacity(cap, inner),
            pos: None,
        }
    }

    /// Unwraps the `BufSeeker`, returning the underlying reader.
    ///
    /// Note that any leftover data in the buffer will be lost.
    pub fn into_inner(self) -> T {
        self.read.into_inner()
    }
}

impl<T: Seek> Seek for BufSeeker<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        use std::mem::transmute;

        struct Repr<T> {
            _inner: T,
            _buf: Vec<u8>,
            pos: usize,
            cap: usize,
        }

        let repr: &mut Repr<T> = unsafe { transmute(&mut self.read) };
        match (&mut self.pos, pos) {
            (&mut Some(ref mut self_pos), SeekFrom::Current(pos)) if (pos < 0 && -pos as u64 <= repr.pos as u64) || pos < (repr.cap - repr.pos) as i64 => {
                if pos < 0 {
                    repr.pos -= -pos as usize;
                    *self_pos -= -pos as u64;
                } else {
                    repr.pos += pos as usize;
                    *self_pos += pos as u64;
                }

                Ok(*self_pos)
            },
            (self_pos, pos) => {
                let pos = try!(self.read.seek(pos));
                repr.pos = 0;
                repr.cap = 0;
                *self_pos = Some(pos);
                Ok(pos)
            },
        }
    }
}

impl<T: Read> Read for BufSeeker<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = try!(self.read.read(buf));
        self.pos.as_mut().map(|v| *v += read as u64);
        Ok(read)
    }
}

impl<T: BufRead> BufRead for BufSeeker<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.read.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.read.consume(amt);
        self.pos.as_mut().map(|v| *v += amt as u64);
    }
}
