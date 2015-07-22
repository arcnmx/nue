use std::mem::{size_of, transmute};

use self::unstable::{repr, box_from, box_into};

/// A marker trait indicating that a type is Plain Old Data.
///
/// It is unsafe to `impl` this manually, use `#[derive(Pod)]` instead.
pub unsafe trait Pod: Sized {
    #[doc(hidden)]
    fn __assert_pod() { }
}

/// Helper methods for converting POD types to/from byte slices and vectors
pub trait PodExt: Sized {
    /// Borrows the POD as a byte slice
    fn as_slice<'a>(&'a self) -> &'a [u8] {
        use std::slice::from_raw_parts;

        unsafe { from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }

    /// Borrows the POD as a mutable byte slice
    fn mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        use std::slice::from_raw_parts_mut;

        unsafe { from_raw_parts_mut(self as *mut Self as *mut u8, size_of::<Self>()) }
    }

    /// Borrows a new instance of the POD from a byte slice
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    fn from_slice<'a>(slice: &'a [u8]) -> &'a Self {
        let slice = repr(slice);
        assert_eq!(slice.len, size_of::<Self>());
        unsafe { &*(slice.data as *const _) }
    }

    /// Borrows a mutable instance of the POD from a mutable byte slice
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    fn from_mut_slice<'a>(slice: &'a mut [u8]) -> &'a mut Self {
        let slice = repr(slice);
        assert_eq!(slice.len, size_of::<Self>());
        unsafe { &mut *(slice.data as *mut _) }
    }

    /// Converts a byte vector to a boxed instance of the POD type
    ///
    /// # Panics
    ///
    /// Panics if `vec.len()` is not the same as the type's size
    fn from_vec(vec: Vec<u8>) -> Box<Self> {
        Self::from_box(vec.into_boxed_slice())
    }

    /// Converts a boxed slice to a boxed instance of the POD type
    ///
    /// # Panics
    ///
    /// Panics if `slice.len()` is not the same as the type's size
    fn from_box(slice: Box<[u8]>) -> Box<Self> {
        assert!(slice.len() == size_of::<Self>());
        unsafe {
            box_from(repr(&*box_into(slice)).data as *mut _)
        }
    }

    /// Converts a POD type from one to another of the same size.
    ///
    /// # Panics
    ///
    /// Panics if the two types are not the same size
    fn map<'a, T: Pod>(&'a self) -> &'a T {
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
    fn map_mut<'a, T: Pod>(&'a mut self) -> &'a mut T {
        assert_eq!(size_of::<Self>(), size_of::<T>());
        unsafe {
            transmute(self)
        }
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
    ($([T; $x:expr]),*) => {
        $(
            unsafe impl<T: Pod> Pod for [T; $x] { }
        )*
    };
}

unsafe impl<T: Pod> Pod for (T,) { }
pod_def! { [T; 0], [T; 1], [T; 2], [T; 3], [T; 4], [T; 5], [T; 6], [T; 7], [T; 8], [T; 9] }
pod_def! { [T; 10], [T; 11], [T; 12], [T; 13], [T; 14], [T; 15], [T; 16], [T; 17], [T; 18], [T; 19] }
pod_def! { [T; 20], [T; 21], [T; 22], [T; 23], [T; 24], [T; 25], [T; 26], [T; 27], [T; 28], [T; 29] }
pod_def! { [T; 30], [T; 31], [T; 32] }

#[cfg(feature = "unstable")]
mod unstable {
    use std::raw::{Repr, Slice};

    pub fn repr<T>(t: &[T]) -> Slice<T> { t.repr() }

    pub unsafe fn box_from<T>(raw: *mut T) -> Box<T> { Box::from_raw(raw) }
    pub fn box_into<T: ?Sized>(b: Box<T>) -> *mut T { Box::into_raw(b) }
}

#[cfg(not(feature = "unstable"))]
mod unstable {
    use std::mem::transmute;

    pub struct Slice<T> {
        pub data: *const T,
        pub len: usize,
    }

    pub fn repr<T>(t: &[T]) -> Slice<T> { unsafe { transmute(t) } }

    pub unsafe fn box_from<T>(raw: *mut T) -> Box<T> { transmute(raw) }
    pub fn box_into<T: ?Sized>(b: Box<T>) -> *mut T { unsafe { transmute(b) } }
}
