use std::io::{self, Read, Write, BufRead, Seek, SeekFrom, copy, sink, repeat};
use std::cmp::min;

/// Extension trait that wraps a stream in a new forward seeker.
pub trait SeekForward<T> {
    /// Creates a new `SeekForward*` wrapper around the underlying stream.
    fn seek_forward(self) -> T;
}

/// A limited form of seeking that can only be reset from the beginning.
///
/// Useful for expressing compressed or cipher streams.
pub trait SeekRewind {
    /// Seeks back to the beginning of the stream.
    /// Conceptually equivalent to `seek(SeekFrom::Start(0))`.
    fn rewind(&mut self) -> io::Result<()>;
}

impl<T: Seek> SeekRewind for T {
    fn rewind(&mut self) -> io::Result<()> {
        self.seek(SeekFrom::Start(0)).map(|_| ())
    }
}

/// A forward seeking wrapper around a `Read` type.
pub struct SeekForwardRead<T> {
    stream: T,
    pos: u64,
}

/// A forward seeking wrapper around a `Read + SeekRewind` type.
pub struct SeekForwardRewind<T> {
    stream: T,
    pos: u64,
}

/// A forward seeking wrapper around a `Write` type.
pub struct SeekForwardWrite<T> {
    stream: T,
    pos: u64,
}

fn forward_offset(stream_pos: u64, seek: SeekFrom) -> Option<u64> {
    match seek {
        SeekFrom::Start(pos) if pos >= stream_pos => Some(pos - stream_pos),
        SeekFrom::Current(pos) if pos >= 0 => Some(pos as u64),
        _ => None,
    }
}

impl<T: Read> Seek for SeekForwardRead<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let offset = try!(forward_offset(self.pos, pos)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid forward seek"))
        );

        self.pos += try!(copy(&mut self.take(offset), &mut sink()));

        Ok(self.pos)
    }
}

impl<T: Read + SeekRewind> Seek for SeekForwardRewind<T> {
    fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
        let offset = match forward_offset(self.pos, seek) {
            Some(pos) => pos,
            None => match seek {
                SeekFrom::Start(pos) => {
                    try!(self.stream.rewind());
                    pos
                },
                SeekFrom::Current(pos) if pos < 0 => {
                    try!(self.stream.rewind());
                    let pos = -pos as u64;
                    (self.pos - min(self.pos, pos)) as u64
                },
                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid forward seek"))
            }
        };

        self.pos += try!(copy(&mut self.take(offset), &mut sink()));

        Ok(self.pos)
    }
}

impl<T: Write> Seek for SeekForwardWrite<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let offset = try!(forward_offset(self.pos, pos)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid forward seek"))
        );

        self.pos += try!(copy(&mut repeat(0).take(offset), self));

        Ok(self.pos)
    }
}

impl<T: Read> Read for SeekForwardRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = try!(self.stream.read(buf));
        self.pos += read as u64;
        Ok(read)
    }
}

impl<T: BufRead> BufRead for SeekForwardRead<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
        self.stream.consume(amt);
    }
}

impl<T: Read> Read for SeekForwardRewind<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = try!(self.stream.read(buf));
        self.pos += read as u64;
        Ok(read)
    }
}

impl<T: BufRead> BufRead for SeekForwardRewind<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
        self.stream.consume(amt);
    }
}

impl<T: Write> Write for SeekForwardWrite<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let write = try!(self.stream.write(buf));
        self.pos += write as u64;
        Ok(write)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl<T: Read> SeekForward<SeekForwardRead<T>> for T {
    fn seek_forward(self) -> SeekForwardRead<T> {
        SeekForwardRead {
            stream: self,
            pos: 0,
        }
    }
}

impl<T: SeekRewind> SeekForward<SeekForwardRewind<T>> for T {
    fn seek_forward(self) -> SeekForwardRewind<T> {
        SeekForwardRewind {
            stream: self,
            pos: 0,
        }
    }
}

impl<T: Write> SeekForward<SeekForwardWrite<T>> for T {
    fn seek_forward(self) -> SeekForwardWrite<T> {
        SeekForwardWrite {
            stream: self,
            pos: 0,
        }
    }
}
