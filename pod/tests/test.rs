extern crate pod;

use pod::{Pod, Le, Be, Encode, Decode};
use pod::packed::{Packed, Aligned, Un};
use std::io::{Cursor, Seek, SeekFrom};

#[cfg(not(feature = "unstable"))]
mod stable {
    use pod::packed::Unaligned;
    unsafe impl Unaligned for super::POD { }
}

unsafe impl Packed for POD { }
unsafe impl Pod for POD { }

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
struct POD {
    zero: u8,
    ffff: Un<u16>,
    one: Le<u16>,
    two: Be<u32>,
}

const POD_BYTES: [u8; 9] = [
    0x00,
    0xff, 0xff,
    0x01, 0x00,
    0x00, 0x00, 0x00, 0x02,
];

fn sample() -> POD {
    POD {
        zero: 0,
        ffff: 0xffffu16.unaligned(),
        one: Le::new(1),
        two: Be::new(2),
    }
}

#[test]
fn pod_encoding() {
    let pod = sample();

    let buffer = Vec::new();
    let mut buffer = Cursor::new(buffer);

    pod.encode(&mut buffer).unwrap();

    buffer.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(pod, POD::decode(&mut buffer).unwrap());

    let buffer = buffer.into_inner();

    assert_eq!(&buffer[..], &POD_BYTES[..]);
}

#[test]
fn pod_slice() {
    assert_eq!(sample(), *POD::from_slice(&POD_BYTES))
}

#[test]
fn pod_box() {
    use std::iter::FromIterator;

    let vec: Vec<u8> = Vec::from_iter(POD_BYTES.iter().cloned());
    let boxed = POD::from_vec(vec);
    assert_eq!(*boxed, sample());
}
