#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(nue_macros)]

extern crate nue_io;
extern crate pod;

use std::io::Cursor;
use std::mem::size_of;
use pod::{Encode, Decode};

#[derive(NueEncode, NueDecode, PartialEq, Debug)]
struct _PodTest;

#[test]
fn encode_decode() {
    #[derive(PodReprPacked)]
    struct POD1 {
        _0: u8,
        _1: u16,
    }

    const UNIT: &'static () = &();

    #[derive(NueEncode, NueDecode, PartialEq, Debug)]
    struct POD2 {
        _0: u8,
        #[nue(align = "1", skip = "0", limit = "2", consume = "true", cond = "self._0 == 1")]
        _1: u16,
        #[nue(cond = "false", default = "UNIT")]
        _2: &'static (),
        _3: (),
    }

    let pod1 = POD1 { _0: 1, _1: 2 };
    let pod2 = POD2 { _0: 1, _1: 2, _2: UNIT, _3: () };

    let buffer1 = Vec::new();
    let mut buffer1 = Cursor::new(buffer1);
    pod1.encode(&mut buffer1).unwrap();

    let buffer2 = Vec::new();
    let mut buffer2 = Cursor::new(buffer2);
    pod2.encode(&mut buffer2).unwrap();

    let buffer1 = buffer1.into_inner();
    let buffer2 = buffer2.into_inner();

    assert_eq!(size_of::<POD1>(), 3);
    assert_eq!(&buffer1, &buffer2);

    let mut buffer2 = Cursor::new(buffer2);
    let pod2_decoded = Decode::decode(&mut buffer2).unwrap();
    assert_eq!(&pod2, &pod2_decoded);
}
