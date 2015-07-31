use std::io::{self, Read, Write, BufReader, BufRead, Cursor};
use std::ffi::{CString, CStr};
use ::Pod;

use uninitialized::UNINITIALIZED;
use nue_io::ReadExactExt;

/// Encodes an value's binary representation to a `Write`.
///
/// Note that this is not serialization, and some data may be lost.
/// It is left up to the implementation to decide what its representation is.
///
/// # Provided Implementations
///
/// ## `String`, `str`
///
/// `String`s are encoded as their raw UTF-8 representation. Length is not encoded, and
/// must be provided when decoding (or reads to end of stream). Will error upon invalid UTF-8
/// sequences, nul bytes, etc.
///
/// ## `CString`, `CStr`
///
/// `CString`s are encoded with a trailing nul byte. Decoding runs until the first nul byte reached.
///
/// ## `Vec<T>`, `[T]`
///
/// Vectors are encoded in sequence as packed raw values. Length is not encoded.
///
/// ## `Option<T>`
///
/// `Some(T)` is encoded as `T`, `None` writes 0 bytes. Decoding will always produce `Some(T)`
///
/// ## `Pod`
///
/// `Pod` types are encoded as their raw in-memory representation. Use `EndianPrimitive` members to
/// control the byte order.
pub trait Encode {
    /// Options that may be provided to affect how the value is encoded
    type Options: Default;

    /// Encodes to the `Write` with default options
    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> { self.encode_options(w, Default::default()) }

    /// Encodes to the `Write` with the provided options
    fn encode_options<W: Write>(&self, w: &mut W, _options: Self::Options) -> io::Result<()> { self.encode(w) }

    /// Encodes to a new byte vector
    fn encode_vec(&self) -> io::Result<Vec<u8>> {
        let data = Vec::new();
        let mut cursor = Cursor::new(data);

        try!(self.encode(&mut cursor));

        Ok(cursor.into_inner())
    }

    /// Encodes to a new byte vector with the provided options
    fn encode_vec_options(&self, options: Self::Options) -> io::Result<Vec<u8>> {
        let data = Vec::new();
        let mut cursor = Cursor::new(data);

        try!(self.encode_options(&mut cursor, options));

        Ok(cursor.into_inner())
    }
}

/// Decodes data from a `Read` into a new value.
///
/// See `Encode` for more details.
pub trait Decode: Sized {
    /// Options will affect how the value is decoded, and may be used to provide any data that
    /// is normally lost during encoding.
    type Options: Default;

    /// Decodes from the `Read` with default options
    fn decode<R: Read>(r: &mut R) -> io::Result<Self> { Self::decode_options(r, Default::default()) }

    /// Decodes from the `Read` with the provided options
    fn decode_options<R: Read>(r: &mut R, _options: Self::Options) -> io::Result<Self> { Self::decode(r) }

    /// Decodes from a byte slice
    fn decode_slice(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);

        Self::decode(&mut cursor)
    }

    /// Decodes from a byte slice with the provided options
    fn decode_slice_options(data: &[u8], options: Self::Options) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);

        Self::decode_options(&mut cursor, options)
    }

    /// Implement to assert that the decoded contents are valid
    ///
    /// # Warning
    ///
    /// This takes `&self`, meaning the object is still created before validation occurs.
    /// Therefore any `Drop` implementations must not assume that the object is valid.
    fn validate(&self) -> io::Result<()> { Ok(()) }
}

impl<T: Encode> Encode for Option<T> {
    type Options = T::Options;

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        self.as_ref().map(|v| v.encode(w)).unwrap_or(Ok(()))
    }

    fn encode_options<W: Write>(&self, w: &mut W, options: Self::Options) -> io::Result<()> {
        self.as_ref().map(|v| v.encode_options(w, options)).unwrap_or(Ok(()))
    }
}

impl<T: Decode> Decode for Option<T> {
    type Options = T::Options;

    fn decode<R: Read>(r: &mut R) -> io::Result<Self> {
        T::decode(r).map(Some)
    }

    fn decode_options<R: Read>(r: &mut R, options: Self::Options) -> io::Result<Self> {
        T::decode_options(r, options).map(Some)
    }
}

impl<T: Pod> Encode for T {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.as_slice())
    }
}

impl<T: Pod> Decode for T {
    type Options = ();

    fn decode<R: Read>(r: &mut R) -> io::Result<Self> {
        // TODO: Would be nice if we could use [0u8; size_of::<T>()]
        let mut pod: Self = Pod::zeroed();

        try!(r.read_exact(pod.mut_slice()));
        Ok(pod)
    }
}

impl Decode for String {
    type Options = StringDecodeOptions;

    fn decode_options<R: Read>(r: &mut R, options: Self::Options) -> io::Result<Self> {
        if let Some(len) = options.len {
            let mut vec = if UNINITIALIZED {
                let mut vec = Vec::with_capacity(len);
                unsafe { vec.set_len(len); }
                vec
            } else {
                vec![0; len]
            };
            try!(r.read_exact(&mut vec[..]));
            String::from_utf8(vec).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        } else {
            let mut string = String::new();
            try!(r.read_to_string(&mut string));
            Ok(string)
        }
    }
}

impl Encode for String {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (**self).encode(w)
    }
}

impl Encode for str {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.as_bytes())
    }
}

impl<'a> Encode for &'a str {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (*self).encode(w)
    }
}

impl Decode for CString {
    type Options = CStringDecodeOptions;

    fn decode_options<R: Read>(r: &mut R, options: Self::Options) -> io::Result<Self> {
        if options.require_nul {
            unimplemented!()
        }

        unsafe {
            let mut err = Ok(());
            let string = CString::from_vec_unchecked(r.bytes().map(|c| match c {
                Ok(c) => c,
                Err(e) => {
                    err = Err(e);
                    0
                },
            }).take_while(|&c| c > 0).collect());
            err.map(|_| string)
        }
    }
}

impl Encode for CString {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (**self).encode(w)
    }
}

impl Encode for CStr {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(self.to_bytes_with_nul())
    }
}

impl<'a> Encode for &'a CStr {
    type Options = ();

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (*self).encode(w)
    }
}

impl<T: Decode> Decode for Vec<T> where T::Options: Clone {
    type Options = VecDecodeOptions<T::Options>;

    fn decode_options<R: Read>(r: &mut R, options: Self::Options) -> io::Result<Self> {
        let mut vec = Vec::with_capacity(options.len.unwrap_or(0));
        if let Some(len) = options.len {
            for _ in 0..len {
                vec.push(try!(T::decode_options(r, options.options.clone())));
            }
        } else {
            let r = &mut BufReader::new(r);
            while try!(r.fill_buf()).len() > 0 {
                vec.push(try!(T::decode_options(r, options.options.clone())));
            }
        }

        Ok(vec)
    }
}

impl<T: Encode> Encode for Vec<T> where T::Options: Clone {
    type Options = T::Options;

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (**self).encode(w)
    }

    fn encode_options<W: Write>(&self, w: &mut W, options: Self::Options) -> io::Result<()> {
        (**self).encode_options(w, options)
    }
}

impl<T: Encode> Encode for [T] where T::Options: Clone {
    type Options = T::Options;

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        for ref v in self {
            try!(v.encode(w));
        }

        Ok(())
    }

    fn encode_options<W: Write>(&self, w: &mut W, options: Self::Options) -> io::Result<()> {
        for ref v in self {
            try!(v.encode_options(w, options.clone()));
        }

        Ok(())
    }
}

impl<'a, T: Encode> Encode for &'a [T] where T::Options: Clone {
    type Options = T::Options;

    fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        (*self).encode(w)
    }

    fn encode_options<W: Write>(&self, w: &mut W, options: Self::Options) -> io::Result<()> {
        (*self).encode_options(w, options)
    }
}

/// Describes how to decode a `Vec<T>`
#[derive(Clone, Default, Debug)]
pub struct VecDecodeOptions<T> {
    /// Reads `Some(len)` items, or until EOF
    pub len: Option<usize>,

    /// The options used to decode each individual item
    pub options: T,
}

/// Describes how to decode a `String`
#[derive(Copy, Clone, Default, Debug)]
pub struct StringDecodeOptions {
    /// Reads `Some(len)` bytes, or until EOF
    pub len: Option<usize>,
}

/// Describes how to decode a `CString`
#[derive(Copy, Clone, Default, Debug)]
pub struct CStringDecodeOptions {
    /// When true, errors if EOF is reached before a nul byte is found
    pub require_nul: bool,
}
