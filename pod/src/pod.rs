use std::mem::size_of;

use self::unstable::{repr, box_from, box_into};

/// A marker trait indicating that a type is Plain Old Data.
///
/// `impl` this on POD structs to access the slice helper methods of `PodExt`
/// and I/O helpers `Encode` and `Decode`.
/// 
/// `#[repr(packed)]` and `#[repr(C)]` may be helpful for consistent representation across
/// platforms.
pub trait Pod: Sized { }

/// An unsafe marker trait that indicates a type can be used as Plain Old Data.
///
/// Should be used very carefully to mark a type as POD when it contains a non-POD
/// member such as a reference.
///
/// ## Nightly / Unstable
///
/// The `unstable` cargo flag will automatically implement this trait on all applicable types.
/// Manual unsafe implementation is currently required on the stable and beta channels.
pub unsafe trait PodType: Pod { }

/// Helper methods for converting Plain Old Data types to/from byte slices and vectors
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
}

impl<T: PodType> PodExt for T { }

impl Pod for () { }
impl Pod for f32 { }
impl Pod for f64 { }
impl Pod for i8 { }
impl Pod for u8 { }
impl Pod for i16 { }
impl Pod for u16 { }
impl Pod for i32 { }
impl Pod for u32 { }
impl Pod for i64 { }
impl Pod for u64 { }
impl Pod for isize { }
impl Pod for usize { }
impl<T> Pod for *const T { }
impl<T> Pod for *mut T { }

macro_rules! pod_def {
    ($([T; $x:expr]),*) => {
        $(
            impl<T: Pod> Pod for [T; $x] { }
        )*
    };
    ($($t:ident),*) => {
        impl<$($t: Pod),*> Pod for ($($t),*,) { }
    };
}

pod_def! { A }
pod_def! { A, B }
pod_def! { A, B, C }
pod_def! { A, B, C, D }
pod_def! { A, B, C, D, E }
pod_def! { A, B, C, D, E, F }
pod_def! { A, B, C, D, E, F, G }
pod_def! { A, B, C, D, E, F, G, H }
pod_def! { A, B, C, D, E, F, G, H, I }
pod_def! { A, B, C, D, E, F, G, H, I, J }
pod_def! { A, B, C, D, E, F, G, H, I, J, K }
pod_def! { A, B, C, D, E, F, G, H, I, J, K, L }
pod_def! { [T; 0], [T; 1], [T; 2], [T; 3], [T; 4], [T; 5], [T; 6], [T; 7], [T; 8], [T; 9] }
pod_def! { [T; 10], [T; 11], [T; 12], [T; 13], [T; 14], [T; 15], [T; 16], [T; 17], [T; 18], [T; 19] }
pod_def! { [T; 20], [T; 21], [T; 22], [T; 23], [T; 24], [T; 25], [T; 26], [T; 27], [T; 28], [T; 29] }
pod_def! { [T; 30], [T; 31], [T; 32] }

#[cfg(feature = "unstable")]
mod unstable {
    use std::raw::{Repr, Slice};

    unsafe impl super::PodType for .. { }

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
