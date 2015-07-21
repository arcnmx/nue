use std::io::{self, Seek, SeekFrom};

/// An extension trait that will seek to meet a specified alignment.
pub trait SeekAlignExt {
    /// Seeks forward to a multiple of `alignment`.
    ///
    /// Returns the resulting offset in the stream upon success.
    fn align_to(&mut self, alignment: u64) -> io::Result<u64>;
}

impl<T: Seek> SeekAlignExt for T {
    fn align_to(&mut self, alignment: u64) -> io::Result<u64> {
        let pos = try!(self.seek(SeekFrom::Current(0)));
        let pos_alignment = pos % alignment;
        if pos_alignment > 0 {
            self.seek(SeekFrom::Current((alignment - pos_alignment) as i64))
        } else {
            Ok(pos)
        }
    }
}

#[test]
fn align() {
    use std::io::Cursor;

    let data = [0; 0x80];
    let mut cursor = Cursor::new(&data[..]);

    cursor.seek(SeekFrom::Start(0x25)).unwrap();

    cursor.align_to(0x10).unwrap();
    assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0x30);

    cursor.align_to(0x20).unwrap();
    assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0x40);

    cursor.align_to(0x20).unwrap();
    assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0x40);
}
