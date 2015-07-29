use std::mem::{size_of, transmute, uninitialized};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use packed::{Unaligned, Aligned};
use uninitialized;

use self::unstable::{box_from, box_into};

/// A marker trait indicating that a type is Plain Old Data.
///
/// It is unsafe to `impl` this manually, use `#[derive(Pod)]` instead.
pub unsafe trait Pod: Sized {
    /// Safely borrows the aligned value mutably
    ///
    /// See also: `Aligned::as_aligned_mut`
    #[inline]
    fn mut_aligned<T: Pod + Aligned<Unaligned=Self>>(&mut self) -> Option<&mut T> where Self: Copy + Unaligned {
        unsafe { T::as_aligned_mut(self) }
    }

    /// Safely borrows the unaligned value mutably
    ///
    /// See also: `Aligned::as_unaligned_mut`
    #[inline]
    fn mut_unaligned<T: Copy + Unaligned>(s: &mut T) -> Option<&mut Self> where Self: Aligned<Unaligned=T> {
        unsafe { Self::as_aligned_mut(s) }
    }

    /// Safely converts an unaligned value to its aligned equivalent
    ///
    /// See also: `Aligned::from_unaligned`
    #[inline]
    fn aligned<T: Copy + Unaligned>(s: T) -> Self where Self: Aligned<Unaligned=T> {
        unsafe { Self::from_unaligned(s) }
    }

    #[doc(hidden)]
    fn __assert_pod() { }
}

/// Helper methods for converting POD types to/from byte slices and vectors
pub trait PodExt: Sized {
    /// Borrows the POD as a byte slice
    #[inline]
    fn as_slice<'a>(&'a self) -> &'a [u8] {
        unsafe { from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    /// Borrows the POD as a mutable byte slice
    #[inline]
    fn mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        unsafe { from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }

    /// Borrows a new instance of the POD from a byte slice
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    #[inline]
    fn from_slice<'a>(slice: &'a [u8]) -> &'a Self where Self: Unaligned {
        assert_eq!(slice.len(), size_of::<Self>());
        unsafe { &*(slice.as_ptr() as *const _) }
    }

    /// Borrows a mutable instance of the POD from a mutable byte slice
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    #[inline]
    fn from_mut_slice<'a>(slice: &'a mut [u8]) -> &'a mut Self where Self: Unaligned {
        assert_eq!(slice.len(), size_of::<Self>());
        unsafe { &mut *(slice.as_mut_ptr() as *mut _) }
    }

    /// Converts a byte vector to a boxed instance of the POD type
    ///
    /// # Panics
    ///
    /// Panics if `vec.len()` is not the same as the type's size
    #[inline]
    fn from_vec(vec: Vec<u8>) -> Box<Self> where Self: Unaligned {
        Self::from_box(vec.into_boxed_slice())
    }

    /// Converts a boxed slice to a boxed instance of the POD type
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    #[inline]
    fn from_box(slice: Box<[u8]>) -> Box<Self> where Self: Unaligned {
        assert!(slice.len() == size_of::<Self>());
        unsafe {
            box_from((&mut *box_into(slice)).as_mut_ptr() as *mut _)
        }
    }

    /// Converts a boxed POD to a byte vector
    #[inline]
    fn to_vec(self: Box<Self>) -> Vec<u8> {
        self.to_boxed_slice().into_vec()
    }

    /// Converts a boxed POD to a boxed slice
    #[inline]
    fn to_boxed_slice(self: Box<Self>) -> Box<[u8]> {
        let ptr = box_into(self);
        unsafe {
            let ptr = from_raw_parts_mut(ptr as *mut u8, size_of::<Self>());
            box_from(ptr)
        }
    }

    /// Converts a POD type from one to another of the same size.
    ///
    /// # Panics
    ///
    /// Panics if the two types are not the same size
    #[inline]
    fn map<'a, T: Pod + Unaligned>(&'a self) -> &'a T where Self: Unaligned {
        assert_eq!(size_of::<Self>(), size_of::<T>());
        unsafe {
            transmute(self)
        }
    }

    /// Converts a POD type from one to another of the same size.
    ///
    /// # Panics
    ///
    /// Panics if the two types are not the same size
    #[inline]
    fn map_mut<'a, T: Pod + Unaligned>(&'a mut self) -> &'a mut T where Self: Unaligned {
        assert_eq!(size_of::<Self>(), size_of::<T>());
        unsafe {
            transmute(self)
        }
    }

    /// Generates a new uninitialized instance of a POD type.
    #[inline]
    unsafe fn uninitialized() -> Self {
        uninitialized()
    }

    /// Creates a new zeroed instance of a POD type.
    #[inline]
    fn zeroed() -> Self {
        unsafe { uninitialized::uninitialized() }
    }
}

impl<T: Pod> PodExt for T { }

unsafe impl Pod for () { }
unsafe impl Pod for f32 { }
unsafe impl Pod for f64 { }
unsafe impl Pod for i8 { }
unsafe impl Pod for u8 { }
unsafe impl Pod for i16 { }
unsafe impl Pod for u16 { }
unsafe impl Pod for i32 { }
unsafe impl Pod for u32 { }
unsafe impl Pod for i64 { }
unsafe impl Pod for u64 { }
unsafe impl Pod for isize { }
unsafe impl Pod for usize { }
unsafe impl<T> Pod for *const T { }
unsafe impl<T> Pod for *mut T { }

macro_rules! pod_def {
    ($($x:expr),*) => {
        $(
            unsafe impl<T: Pod> Pod for [T; $x] { }
        )*
    };
}

unsafe impl<T: Pod> Pod for (T,) { }
pod_def! { 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f }
pod_def! { 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f }
pod_def! { 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f }
pod_def! { 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f }
pod_def! { 0x40 }

#[cfg(feature = "unstable")]
mod unstable {
    pub unsafe fn box_from<T: ?Sized>(raw: *mut T) -> Box<T> { Box::from_raw(raw) }
    pub fn box_into<T: ?Sized>(b: Box<T>) -> *mut T { Box::into_raw(b) }
}

#[cfg(not(feature = "unstable"))]
mod unstable {
    use std::mem::transmute;

    pub unsafe fn box_from<T: ?Sized>(raw: *mut T) -> Box<T> { transmute(raw) }
    pub fn box_into<T: ?Sized>(b: Box<T>) -> *mut T { unsafe { transmute(b) } }
}
