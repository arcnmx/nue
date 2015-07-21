use std::marker::PhantomData;
use std::fmt;
use std::hash::{Hash, Hasher};
use byteorder::{ByteOrder, LittleEndian, BigEndian};
use uninitialized::uninitialized;
use pod::{Pod, PodType};

/// A type alias for little endian primitives
pub type Le<T> = EndianPrimitive<LittleEndian, T>;

/// A type alias for big endian primitives
pub type Be<T> = EndianPrimitive<BigEndian, T>;

/// A POD container for a primitive that stores a value in the specified endianness
/// in memory, and transforms on `get`/`set`
pub struct EndianPrimitive<B, T> {
    value: T,
    _phantom: PhantomData<*const B>,
}

impl<B: ByteOrder, T: EndianConvert> EndianPrimitive<B, T> {
    /// Creates a new value
    pub fn new(v: T) -> Self {
        EndianPrimitive {
            value: EndianConvert::to::<B>(v),
            _phantom: PhantomData,
        }
    }

    /// Transforms to the native value
    pub fn get(&self) -> T {
        EndianConvert::from::<B>(&self.value)
    }

    /// Transforms from a native value
    pub fn set(&mut self, v: T) {
        self.value = EndianConvert::to::<B>(v)
    }
}

impl<B, T: Pod> Pod for EndianPrimitive<B, T> { }
unsafe impl<B, T: PodType> PodType for EndianPrimitive<B, T> { }

impl<B: ByteOrder, T: Default + EndianConvert> Default for EndianPrimitive<B, T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<B: ByteOrder, T: EndianConvert> From<T> for EndianPrimitive<B, T> {
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

impl<B, T: fmt::Debug> fmt::Debug for EndianPrimitive<B, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as fmt::Debug>::fmt(&self.value, f)
    }
}

impl<BRHS: ByteOrder, RHS: EndianConvert, B: ByteOrder, T: EndianConvert + PartialEq<RHS>> PartialEq<EndianPrimitive<BRHS, RHS>> for EndianPrimitive<B, T> {
    fn eq(&self, other: &EndianPrimitive<BRHS, RHS>) -> bool {
        self.get().eq(&other.get())
    }
}

impl<B, T: Hash> Hash for EndianPrimitive<B, T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.value.hash(h)
    }
}

impl<B, T: Clone> Clone for EndianPrimitive<B, T> {
    fn clone(&self) -> Self {
        EndianPrimitive {
            value: self.value.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<B, T: Copy> Copy for EndianPrimitive<B, T> { }

/// Describes a value that can be converted to and from a specified byte order.
pub trait EndianConvert {
    /// Converts a value from `B`
    fn from<B: ByteOrder>(&self) -> Self;

    /// Converts a value to `B`
    fn to<B: ByteOrder>(self) -> Self;
}

macro_rules! endian_impl {
    ($t:ty: $s:expr => $r:ident, $w:ident) => {
        impl EndianConvert for $t {
            fn from<B: ByteOrder>(&self) -> Self {
                use std::mem::transmute;
                B::$r(unsafe { transmute::<_, &[u8; $s]>(self) })
            }

            fn to<B: ByteOrder>(self) -> Self {
                use std::mem::transmute;

                unsafe {
                    let mut s = uninitialized();
                    B::$w(transmute::<_, &mut [u8; $s]>(&mut s), self);
                    s
                }
            }
        }
    };
}

endian_impl!(u16: 2 => read_u16, write_u16);
endian_impl!(i16: 2 => read_i16, write_i16);
endian_impl!(i32: 4 => read_i32, write_i32);
endian_impl!(u32: 4 => read_u32, write_u32);
endian_impl!(i64: 8 => read_i64, write_i64);
endian_impl!(u64: 8 => read_u64, write_u64);
endian_impl!(f32: 4 => read_f32, write_f32);
endian_impl!(f64: 8 => read_f64, write_f64);

#[test]
fn endian_size() {
    use std::mem::size_of;
    type B = BigEndian;

    assert_eq!(size_of::<EndianPrimitive<B, i16>>(), 2);
    assert_eq!(size_of::<EndianPrimitive<B, i32>>(), 4);
    assert_eq!(size_of::<EndianPrimitive<B, i64>>(), 8);
    assert_eq!(size_of::<EndianPrimitive<B, f32>>(), 4);
    assert_eq!(size_of::<EndianPrimitive<B, f64>>(), 8);
}
