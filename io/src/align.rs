use std::io;
use seek_forward::{Tell, SeekForward};

/// An extension trait that will seek to meet a specified alignment.
pub trait SeekAlignExt {
    /// Seeks forward to a multiple of `alignment`.
    ///
    /// Returns the resulting offset in the stream upon success.
    fn align_to(&mut self, alignment: u64) -> io::Result<u64>;
}

impl<T: Tell + SeekForward> SeekAlignExt for T {
    fn align_to(&mut self, alignment: u64) -> io::Result<u64> {
        let pos = try!(self.tell());
        let pos_alignment = pos % alignment;
        if pos_alignment > 0 {
            self.seek_forward(alignment - pos_alignment).map(|v| v + pos)
        } else {
            Ok(pos)
        }
    }
}

#[test]
fn align() {
    use std::io::Cursor;
    use seek_forward::{SeekAll, SeekAbsolute};

    let data = [0; 0x80];
    let mut cursor = SeekAll::new(Cursor::new(&data[..]));

    cursor.seek_absolute(0x25).unwrap();

    cursor.align_to(0x10).unwrap();
    assert_eq!(cursor.tell().unwrap(), 0x30);

    cursor.align_to(0x20).unwrap();
    assert_eq!(cursor.tell().unwrap(), 0x40);

    cursor.align_to(0x20).unwrap();
    assert_eq!(cursor.tell().unwrap(), 0x40);
}
