use std::marker::PhantomData;
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use byteorder::{ByteOrder, LittleEndian, BigEndian, NativeEndian};
use uninitialized::uninitialized;
use packed::{Unaligned, Aligned, Packed};
use pod::Pod;

/// A type alias for unaligned little endian primitives
pub type Le<T> = EndianPrimitive<LittleEndian, T>;

/// A type alias for unaligned big endian primitives
pub type Be<T> = EndianPrimitive<BigEndian, T>;

/// A type alias for unaligned native endian primitives
pub type Native<T> = EndianPrimitive<NativeEndian, T>;

/// A POD container for a primitive that stores a value in the specified endianness
/// in memory, and transforms on `get`/`set`
#[repr(C)]
pub struct EndianPrimitive<B, T: EndianConvert> {
    value: T::Unaligned,
    _phantom: PhantomData<*const B>,
}

impl<B: ByteOrder, T: EndianConvert> EndianPrimitive<B, T> {
    /// Creates a new value
    #[inline]
    pub fn new(v: T) -> Self {
        EndianPrimitive {
            value: EndianConvert::to::<B>(v),
            _phantom: PhantomData,
        }
    }

    /// Transforms to the native value
    #[inline]
    pub fn get(&self) -> T {
        EndianConvert::from::<B>(&self.value)
    }

    /// Transforms from a native value
    #[inline]
    pub fn set(&mut self, v: T) {
        self.value = EndianConvert::to::<B>(v)
    }

    /// Gets the inner untransformed value
    #[inline]
    pub fn raw(&self) -> &T::Unaligned {
        &self.value
    }

    /// A mutable reference to the inner untransformed value
    #[inline]
    pub fn raw_mut(&mut self) -> &mut T::Unaligned {
        &mut self.value
    }
}

unsafe impl<B, T: EndianConvert> Pod for EndianPrimitive<B, T> { }
unsafe impl<B, T: EndianConvert> Unaligned for EndianPrimitive<B, T> { }
unsafe impl<B, T: EndianConvert> Packed for EndianPrimitive<B, T> { }

impl<B: ByteOrder, T: Default + EndianConvert> Default for EndianPrimitive<B, T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<B: ByteOrder, T: EndianConvert> From<T> for EndianPrimitive<B, T> {
    #[inline]
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

impl<B: ByteOrder, T: fmt::Debug + EndianConvert> fmt::Debug for EndianPrimitive<B, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as fmt::Debug>::fmt(&self.get(), f)
    }
}

impl<BRHS: ByteOrder, RHS: EndianConvert, B: ByteOrder, T: EndianConvert + PartialEq<RHS>> PartialEq<EndianPrimitive<BRHS, RHS>> for EndianPrimitive<B, T> {
    #[inline]
    fn eq(&self, other: &EndianPrimitive<BRHS, RHS>) -> bool {
        self.get().eq(&other.get())
    }
}

impl<B: ByteOrder, T: EndianConvert + Eq> Eq for EndianPrimitive<B, T> { }

impl<BRHS: ByteOrder, RHS: EndianConvert, B: ByteOrder, T: EndianConvert + PartialOrd<RHS>> PartialOrd<EndianPrimitive<BRHS, RHS>> for EndianPrimitive<B, T> {
    #[inline]
    fn partial_cmp(&self, other: &EndianPrimitive<BRHS, RHS>) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<B: ByteOrder, T: EndianConvert + Ord> Ord for EndianPrimitive<B, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<B, T: EndianConvert + Hash> Hash for EndianPrimitive<B, T> where T::Unaligned: Hash {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.value.hash(h)
    }
}

impl<B, T: EndianConvert> Clone for EndianPrimitive<B, T> {
    #[inline]
    fn clone(&self) -> Self {
        EndianPrimitive {
            value: self.value.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<B, T: EndianConvert> Copy for EndianPrimitive<B, T> { }

/// Describes a value that can be converted to and from a specified byte order.
pub trait EndianConvert: Aligned {
    /// Converts a value from `B`
    fn from<B: ByteOrder>(&Self::Unaligned) -> Self;

    /// Converts a value to `B`
    fn to<B: ByteOrder>(self) -> Self::Unaligned;
}

macro_rules! endian_impl {
    ($t:ty: $s:expr => $r:ident, $w:ident) => {
        impl EndianConvert for $t {
            #[inline]
            fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
                B::$r(s)
            }

            #[inline]
            fn to<B: ByteOrder>(self) -> Self::Unaligned {
                let mut s: Self::Unaligned = unsafe { uninitialized() };
                B::$w(&mut s, self);
                s
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

impl EndianConvert for bool {
    #[inline]
    fn from<B: ByteOrder>(s: &Self::Unaligned) -> Self {
        *s as u8 != 0
    }

    #[inline]
    fn to<B: ByteOrder>(self) -> Self::Unaligned {
        if self as u8 != 0 { true } else { false }
    }
}

#[test]
fn endian_size() {
    use std::mem::size_of;
    use std::mem::align_of;

    type B = NativeEndian;

    assert_eq!(size_of::<EndianPrimitive<B, i16>>(), 2);
    assert_eq!(size_of::<EndianPrimitive<B, i32>>(), 4);
    assert_eq!(size_of::<EndianPrimitive<B, i64>>(), 8);
    assert_eq!(size_of::<EndianPrimitive<B, f32>>(), 4);
    assert_eq!(size_of::<EndianPrimitive<B, f64>>(), 8);

    assert_eq!(align_of::<EndianPrimitive<B, bool>>(), 1);
    assert_eq!(align_of::<EndianPrimitive<B, i16>>(), 1);
    assert_eq!(align_of::<EndianPrimitive<B, i32>>(), 1);
    assert_eq!(align_of::<EndianPrimitive<B, i64>>(), 1);
    assert_eq!(align_of::<EndianPrimitive<B, f32>>(), 1);
    assert_eq!(align_of::<EndianPrimitive<B, f64>>(), 1);
}
