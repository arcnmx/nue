#![feature(custom_attribute, plugin, custom_derive, test)]
#![plugin(nue_macros)]

extern crate test;
extern crate nue;
extern crate pod;

use std::mem::size_of;
use std::io::{BufReader, Cursor, Repeat, Seek, SeekFrom, repeat};
use pod::{Be, Le, PodExt, Decode, Encode};
use test::black_box as bb;
use test::Bencher;

#[derive(Copy, Clone, PodPacked)]
struct DataPod {
    data: [u8; 0x20],
    magic: Be<u32>,
    random: [Le<u64>; 0x10],
}

#[derive(NueEncode, NueDecode)]
struct DataCode {
    data: [u8; 0x20],
    magic: Be<u32>,
    random: [Le<u64>; 0x10],
}

fn repeat_stream() -> BufReader<Repeat> {
    BufReader::new(repeat(0x5a))
}

fn memory_stream() -> Cursor<Vec<u8>> {
    Cursor::new(vec![0x5a; 0x200])
}

fn data_pod() -> DataPod {
    DataPod {
        data: [0x5a; 0x20],
        magic: Be::new(0x5a5a5a5a),
        random: [Le::new(0x5a5a5a5a5a5a5a5a); 0x10],
    }
}

fn data_code() -> DataCode {
    DataCode {
        data: [0x5a; 0x20],
        magic: Be::new(0x5a5a5a5a),
        random: [Le::new(0x5a5a5a5a5a5a5a5a); 0x10],
    }
}

#[bench]
fn bench_repeat_pod_decode(b: &mut Bencher) {
    let mut stream = repeat_stream();

    b.iter(|| bb(DataPod::decode(&mut stream).unwrap()));
}

#[bench]
fn bench_repeat_decode(b: &mut Bencher) {
    let mut stream = repeat_stream();

    b.iter(|| bb(DataCode::decode(&mut stream).unwrap()));
}

#[bench]
fn bench_pod_decode(b: &mut Bencher) {
    let mut stream = memory_stream();

    b.iter(|| {
        stream.seek(SeekFrom::Start(0)).unwrap();
        bb(DataPod::decode(&mut stream).unwrap())
    });
}

#[bench]
fn bench_decode(b: &mut Bencher) {
    let mut stream = memory_stream();

    b.iter(|| {
        stream.seek(SeekFrom::Start(0)).unwrap();
        bb(DataCode::decode(&mut stream).unwrap())
    });
}

#[bench]
fn bench_pod_decode_ref(b: &mut Bencher) {
    let stream = memory_stream().into_inner();

    b.iter(|| bb(DataPod::from_slice(&stream[..size_of::<DataPod>()])));
}

#[bench]
fn bench_pod_decode_ref_copy(b: &mut Bencher) {
    let stream = memory_stream().into_inner();

    b.iter(|| bb(*DataPod::from_slice(&stream[..size_of::<DataPod>()])));
}

#[bench]
fn bench_pod_encode(b: &mut Bencher) {
    let mut stream = memory_stream();
    let data = data_pod();

    b.iter(|| {
        stream.seek(SeekFrom::Start(0)).unwrap();
        data.encode(&mut stream).unwrap();
        bb(&stream);
    });
}

#[bench]
fn bench_encode(b: &mut Bencher) {
    let mut stream = memory_stream();
    let data = data_code();

    b.iter(|| {
        stream.seek(SeekFrom::Start(0)).unwrap();
        data.encode(&mut stream).unwrap();
        bb(&stream);
    });
}

#[bench]
fn bench_pod_encode_ref(b: &mut Bencher) {
    let mut stream = memory_stream().into_inner();
    let data = data_pod();

    b.iter(|| {
        let mem = PodExt::from_mut_slice(&mut stream[..size_of::<DataPod>()]);
        *mem = data;
        bb(mem);
    });
}
