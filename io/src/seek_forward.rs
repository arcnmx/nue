use std::io::{self, Read, Write, BufRead, Seek, SeekFrom, copy, sink, repeat};

/// A limited form of seeking that can only be reset from the beginning.
///
/// Useful for expressing compressed or cipher streams.
pub trait SeekRewind {
    /// Seeks back to the beginning of the stream.
    ///
    /// Conceptually equivalent to `seek(SeekFrom::Start(0))`.
    fn seek_rewind(&mut self) -> io::Result<()>;
}

/// A limited form of seeking that only allows seeking forward.
pub trait SeekForward {
    /// Seeks forward in the stream.
    ///
    /// Returns the number of bytes skipped.
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64>;
}

/// A limited form of seeking that only allows seeking backward.
pub trait SeekBackward {
    /// Seeks backward in the stream.
    ///
    /// Returns the number of bytes reversed by.
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64>;
}

/// A limited form of seeking that only seeks to an offset from the end of the stream.
pub trait SeekEnd {
    /// Seeks to the end of the stream + `offset`.
    ///
    /// Returns the new position in the stream.
    fn seek_end(&mut self, offset: i64) -> io::Result<u64>;
}

/// A limited form of seeking that only seeks to an absolute position from the start.
pub trait SeekAbsolute {
    /// Seeks to the specified position in the stream.
    ///
    /// Returns the new position in the stream.
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64>;
}

/// Exposes the current position in the stream without changing it.
pub trait Tell {
    /// Returns the current absolute position in the stream.
    fn tell(&mut self) -> io::Result<u64>;
}

impl<'a, T: Tell + ?Sized> Tell for &'a mut T {
    #[inline]
    fn tell(&mut self) -> io::Result<u64> {
        (**self).tell()
    }
}

impl<'a, T: SeekForward + ?Sized> SeekForward for &'a mut T {
    #[inline]
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        (**self).seek_forward(offset)
    }
}

impl<'a, T: SeekAbsolute + ?Sized> SeekAbsolute for &'a mut T {
    #[inline]
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        (**self).seek_absolute(pos)
    }
}

impl<'a, T: SeekRewind + ?Sized> SeekRewind for &'a mut T {
    #[inline]
    fn seek_rewind(&mut self) -> io::Result<()> {
        (**self).seek_rewind()
    }
}

impl<'a, T: SeekBackward + ?Sized> SeekBackward for &'a mut T {
    #[inline]
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        (**self).seek_backward(offset)
    }
}

impl<'a, T: SeekEnd + ?Sized> SeekEnd for &'a mut T {
    #[inline]
    fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
        (**self).seek_end(offset)
    }
}

impl<T: Tell + ?Sized> Tell for Box<T> {
    #[inline]
    fn tell(&mut self) -> io::Result<u64> {
        (**self).tell()
    }
}

impl<T: SeekForward + ?Sized> SeekForward for Box<T> {
    #[inline]
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        (**self).seek_forward(offset)
    }
}

impl<T: SeekAbsolute + ?Sized> SeekAbsolute for Box<T> {
    #[inline]
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        (**self).seek_absolute(pos)
    }
}

impl<T: SeekRewind + ?Sized> SeekRewind for Box<T> {
    #[inline]
    fn seek_rewind(&mut self) -> io::Result<()> {
        (**self).seek_rewind()
    }
}

impl<T: SeekBackward + ?Sized> SeekBackward for Box<T> {
    #[inline]
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        (**self).seek_backward(offset)
    }
}

impl<T: SeekEnd + ?Sized> SeekEnd for Box<T> {
    #[inline]
    fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
        (**self).seek_end(offset)
    }
}

/// A forward seeking wrapper around a `Read` type.
pub struct SeekForwardRead<T> {
    inner: T,
}

/// A forward seeking wrapper around a `Write` type.
pub struct SeekForwardWrite<T> {
    inner: T,
}

/// An absolute seeking wrapper around a `Tell + SeekForward + SeekRewind` type.
pub struct SeekAbsoluteRewind<T> {
    inner: T,
}

/// A wrapper that decomposes `Seek` into its individual traits.
pub struct SeekAll<T> {
    inner: T,
}

/// A wrapper that implements `Tell` for streams that don't support it.
pub struct ReadWriteTell<T> {
    inner: T,
    pos: u64,
}

impl<T: Read> SeekForward for SeekForwardRead<T> {
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        if offset == 0 {
            Ok(0)
        } else {
            copy(&mut self.inner.by_ref().take(offset), &mut sink())
        }
    }
}

impl<T: Write> SeekForward for SeekForwardWrite<T> {
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        if offset == 0 {
            Ok(0)
        } else {
            copy(&mut repeat(0).take(offset), self)
        }
    }
}

impl<T: Tell + SeekForward + SeekRewind> SeekAbsolute for SeekAbsoluteRewind<T> {
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        if pos == 0 {
            return self.inner.seek_rewind().map(|_| 0);
        }

        let tell = try!(self.inner.tell());
        let tell = if tell > pos {
            try!(self.inner.seek_rewind().map(|_| 0))
        } else {
            tell
        };

        let diff = pos - tell;
        self.inner.seek_forward(diff).map(|v| tell + v)
    }
}

impl<T> Tell for ReadWriteTell<T> {
    #[inline]
    fn tell(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

impl<T: Read> Read for ReadWriteTell<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = try!(self.inner.read(buf));
        self.pos += read as u64;
        Ok(read)
    }
}

impl<T: BufRead> BufRead for ReadWriteTell<T> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
        self.pos += amt as u64;
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        let read = try!(self.inner.read_until(byte, buf));
        self.pos += read as u64;
        Ok(read)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let read = try!(self.inner.read_line(buf));
        self.pos += read as u64;
        Ok(read)
    }
}

impl<T: Write> Write for ReadWriteTell<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let write = try!(self.inner.write(buf));
        self.pos += write as u64;
        Ok(write)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: SeekForward> SeekForward for ReadWriteTell<T> {
    #[inline]
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        let offset = try!(self.inner.seek_forward(offset));
        self.pos += offset;
        Ok(offset)
    }
}

impl<T: SeekRewind> SeekRewind for ReadWriteTell<T> {
    #[inline]
    fn seek_rewind(&mut self) -> io::Result<()> {
        try!(self.inner.seek_rewind());
        self.pos = 0;
        Ok(())
    }
}

impl<T: SeekAbsolute> SeekAbsolute for ReadWriteTell<T> {
    #[inline]
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        let pos = try!(self.inner.seek_absolute(pos));
        self.pos = pos;
        Ok(pos)
    }
}

/*impl<T: Tell + SeekForward + SeekBackward + SeekAbsolute + SeekEnd> Seek for T {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(pos) => self.inner.seek_absolute(pos),
            SeekFrom::Current(offset) if offset > 0 =>
                self.inner.seek_forward(offset as u64).and_then(|_| self.inner.tell()),
            SeekFrom::Current(offset) if offset == 0 => self.inner.tell(),
            SeekFrom::Current(offset) => self.inner.seek_backward(-offset as u64).and_then(|_| self.inner.tell()),
            SeekFrom::End(offset) => self.seek_end(offset),
        }
    }
}*/

/*impl<T: Tell + SeekAbsolute> SeekBackward for _<T> {
    #[inline]
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        let pos = try!(self.inner.tell()).saturating_sub(-offset as u64);
        self.inner.seek_absolute(pos)
    }
}*/

impl<T: Seek> Tell for SeekAll<T> {
    #[inline]
    fn tell(&mut self) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Current(0))
    }
}

impl<T: Seek> SeekForward for SeekAll<T> {
    #[inline]
    fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Current(offset as i64)).map(|_| offset)
    }
}

impl<T: Seek> SeekAbsolute for SeekAll<T> {
    #[inline]
    fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Start(pos))
    }
}

impl<T: Seek> SeekRewind for SeekAll<T> {
    #[inline]
    fn seek_rewind(&mut self) -> io::Result<()> {
        self.inner.seek(SeekFrom::Start(0)).map(|_| ())
    }
}

impl<T: Seek> SeekBackward for SeekAll<T> {
    #[inline]
    fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
        self.inner.seek(SeekFrom::Current(-(offset as i64)))
    }
}

impl<T: Seek> SeekEnd for SeekAll<T> {
    #[inline]
    fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
        self.inner.seek(SeekFrom::End(offset))
    }
}

macro_rules! impl_seek {
    ($t:ident => SeekRewind) => {
        impl<T: SeekRewind> SeekRewind for $t<T> {
            #[inline]
            fn seek_rewind(&mut self) -> io::Result<()> {
                self.inner.seek_rewind()
            }
        }
    };
    ($t:ident => SeekForward) => {
        impl<T: SeekForward> SeekForward for $t<T> {
            #[inline]
            fn seek_forward(&mut self, offset: u64) -> io::Result<u64> {
                self.inner.seek_forward(offset)
            }
        }
    };
    ($t:ident => SeekBackward) => {
        impl<T: SeekBackward> SeekBackward for $t<T> {
            #[inline]
            fn seek_backward(&mut self, offset: u64) -> io::Result<u64> {
                self.inner.seek_backward(offset)
            }
        }
    };
    ($t:ident => SeekAbsolute) => {
        impl<T: SeekAbsolute> SeekAbsolute for $t<T> {
            #[inline]
            fn seek_absolute(&mut self, pos: u64) -> io::Result<u64> {
                self.inner.seek_absolute(pos)
            }
        }
    };
    ($t:ident => SeekEnd) => {
        impl<T: SeekEnd> SeekEnd for $t<T> {
            #[inline]
            fn seek_end(&mut self, offset: i64) -> io::Result<u64> {
                self.inner.seek_end(offset)
            }
        }
    };
    ($t:ident => Tell) => {
        impl<T: Tell> Tell for $t<T> {
            #[inline]
            fn tell(&mut self) -> io::Result<u64> {
                self.inner.tell()
            }
        }
    };
    ($t:ident => Read) => {
        impl<T: Read> Read for $t<T> {
            #[inline]
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.inner.read(buf)
            }
        }
    };
    ($t:ident => Write) => {
        impl<T: Write> Write for $t<T> {
            #[inline]
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.inner.write(buf)
            }

            #[inline]
            fn flush(&mut self) -> io::Result<()> {
                self.inner.flush()
            }
        }
    };
    ($t:ident => BufRead) => {
        impl<T: BufRead> BufRead for $t<T> {
            #[inline]
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                self.inner.fill_buf()
            }

            #[inline]
            fn consume(&mut self, amt: usize) {
                self.inner.consume(amt)
            }
        }
    };
}

impl_seek!(SeekForwardRead => SeekRewind);
impl_seek!(SeekForwardRead => Tell);
impl_seek!(SeekForwardRead => SeekAbsolute);
impl_seek!(SeekForwardRead => SeekEnd);
impl_seek!(SeekForwardRead => SeekBackward);
impl_seek!(SeekForwardRead => BufRead);
impl_seek!(SeekForwardRead => Read);

impl_seek!(SeekForwardWrite => SeekRewind);
impl_seek!(SeekForwardWrite => Tell);
impl_seek!(SeekForwardWrite => SeekAbsolute);
impl_seek!(SeekForwardWrite => SeekEnd);
impl_seek!(SeekForwardWrite => SeekBackward);
impl_seek!(SeekForwardWrite => Write);

impl_seek!(SeekAbsoluteRewind => SeekRewind);
impl_seek!(SeekAbsoluteRewind => Tell);
impl_seek!(SeekAbsoluteRewind => SeekForward);
impl_seek!(SeekAbsoluteRewind => SeekEnd);
impl_seek!(SeekAbsoluteRewind => SeekBackward);
impl_seek!(SeekAbsoluteRewind => Read);
impl_seek!(SeekAbsoluteRewind => Write);
impl_seek!(SeekAbsoluteRewind => BufRead);

impl_seek!(SeekAll => BufRead);
impl_seek!(SeekAll => Read);
impl_seek!(SeekAll => Write);

impl<T> SeekForwardRead<T> {
    /// Creates a new `SeekForwardRead`.
    pub fn new(inner: T) -> Self {
        SeekForwardRead {
            inner: inner,
        }
    }
}

impl<T> SeekForwardWrite<T> {
    /// Creates a new `SeekForwardWrite`.
    pub fn new(inner: T) -> Self {
        SeekForwardWrite {
            inner: inner,
        }
    }
}

impl<T> SeekAbsoluteRewind<T> {
    /// Creates a new `SeekAbsoluteRewind`.
    pub fn new(inner: T) -> Self {
        SeekAbsoluteRewind {
            inner: inner,
        }
    }
}

impl<T> ReadWriteTell<T> {
    /// Creates a new `ReadWriteTell`.
    pub fn new(inner: T) -> Self {
        ReadWriteTell {
            inner: inner,
            pos: 0,
        }
    }
}

impl<T> SeekAll<T> {
    /// Creates a new `SeekAll`.
    pub fn new(inner: T) -> Self {
        SeekAll {
            inner: inner,
        }
    }
}
