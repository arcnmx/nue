#![deny(missing_docs)]
#![cfg_attr(feature = "unstable", feature(optin_builtin_traits))]

//! A safe approach to using `#[repr(packed)]` data.
//!
//! See `nue_macros` for the automagic `#[packed]` attribute.

use std::mem::{transmute, replace, uninitialized, forget};
use std::marker::PhantomData;

use std::mem::align_of;

/// A marker trait indicating that a type has an alignment of `1`.
///
/// In general, only applies to `()`, `bool`, `i8`, `u8`, and any types that
/// contain only members of these types.
pub unsafe trait Unaligned { }

/// A type alias that represents the unaligned type of `T`.
pub type Un<T> = <T as Aligned>::Unaligned;

/// A marker trait indicating that a type has an alignment over `1`,
/// and is therefore not safe to use in an unaligned context.
pub unsafe trait Aligned: Sized {
    /// An unaligned representation of this type. Usually a u8 array of the
    /// same size.
    type Unaligned: Unaligned + Sized + Copy;

    /// Determines whether an unaligned representation of this type is aligned.
    #[inline]
    fn is_aligned(u: &Self::Unaligned) -> bool {
        u as *const _ as usize % align_of::<Self>() == 0
    }

    /// Borrows the value as unaligned.
    #[inline]
    fn as_unaligned(&self) -> &Self::Unaligned {
        unsafe { transmute(self) }
    }

    /// Mutably borrows the value as unaligned.
    #[inline]
    unsafe fn as_unaligned_mut(&mut self) -> &mut Self::Unaligned {
        transmute(self)
    }

    /// Borrows an unaligned type as an aligned value.
    ///
    /// Returns `None` if `u` is not aligned.
    #[inline]
    fn as_aligned(u: &Self::Unaligned) -> Option<&Self> {
        if Self::is_aligned(u) {
            Some(unsafe { Self::as_aligned_unchecked(u) })
        } else {
            None
        }
    }

    /// Mutably borrows an unaligned type as an aligned value.
    ///
    /// Returns `None` if `u` is not aligned.
    #[inline]
    unsafe fn as_aligned_mut(u: &mut Self::Unaligned) -> Option<&mut Self> {
        if Self::is_aligned(u) {
            Some(Self::as_aligned_mut_unchecked(u))
        } else {
            None
        }
    }

    /// Borrows an unaligned type as an aligned value, without first checking the alignment.
    ///
    /// Causes undefined behaviour if used improprly!
    #[inline]
    unsafe fn as_aligned_unchecked(u: &Self::Unaligned) -> &Self {
        transmute(u)
    }

    /// Mutably borrows an unaligned type as an aligned value, without first checking the alignment.
    ///
    /// Causes undefined behaviour if used improprly!
    #[inline]
    unsafe fn as_aligned_mut_unchecked(u: &mut Self::Unaligned) -> &mut Self {
        transmute(u)
    }

    /// Converts a value to its unaligned representation.
    #[inline]
    fn unaligned(self) -> Self::Unaligned {
        unsafe {
            let mut s: Self::Unaligned = uninitialized();
            forget(replace(&mut s, *transmute::<_, &Self::Unaligned>(&self)));
            s
        }
    }

    /// Copies a value from its unaligned representation.
    #[inline]
    unsafe fn from_unaligned(u: Self::Unaligned) -> Self {
        let mut s: Self = uninitialized();
        forget(replace(s.as_unaligned_mut(), u));
        s
    }

    #[doc(hidden)]
    unsafe fn __assert_unaligned() { }
}

/// A marker trait indicating that a type is `#[repr(packed)]`.
///
/// This means that all its members are packed or have an alignment of `1`,
/// and its memory layout is guaranteed to be in member declaration order.
pub unsafe trait Packed: Unaligned {
    #[doc(hidden)]
    fn __assert_unaligned() { }
}

#[cfg(feature = "unstable")]
mod impls {
    use super::Unaligned;

    unsafe impl Unaligned for .. { }

    // All primitives except the packed ones listed below
    impl !Unaligned for char { }
    impl !Unaligned for f32 { }
    impl !Unaligned for f64 { }
    impl !Unaligned for i16 { }
    impl !Unaligned for u16 { }
    impl !Unaligned for i32 { }
    impl !Unaligned for u32 { }
    impl !Unaligned for i64 { }
    impl !Unaligned for u64 { }
    impl !Unaligned for isize { }
    impl !Unaligned for usize { }
    impl<T> !Unaligned for *const T { }
    impl<T> !Unaligned for *mut T { }
    impl<'a, T> !Unaligned for &'a T { }
    impl<'a, T> !Unaligned for &'a mut T { }
}

#[cfg(not(feature = "unstable"))]
mod impls {
    use super::Unaligned;

    unsafe impl Unaligned for () { }
    unsafe impl Unaligned for i8 { }
    unsafe impl Unaligned for u8 { }
    unsafe impl Unaligned for bool { }
}

unsafe impl<T> Unaligned for PhantomData<T> { }

macro_rules! aligned_assert {
    ($t:ident) => {
        unsafe fn __assert_unaligned() {
            ::std::mem::forget(::std::mem::transmute::<$t, $t::Unaligned>(::std::mem::uninitialized()));
        }
    };
}

macro_rules! aligned_self {
    ($t:ty) => {
        unsafe impl Aligned for $t {
            type Unaligned = $t;
        }
    };
    ($($t:ty),*) => {
        $(
            aligned_self!($t);
        )*
    };
}

macro_rules! aligned_impl {
    ($t:ident: $s:expr) => {
        unsafe impl Aligned for $t {
            type Unaligned = [u8; $s];

            aligned_assert!(Self);
        }
    };
    ($($t:ident: $e:expr),*) => {
        $(
            aligned_impl!($t: $e);
        )*
    };
}

aligned_impl! {
    char: 4,
    f32: 4,
    f64: 8,
    i16: 2,
    u16: 2,
    i32: 4,
    u32: 4,
    i64: 8,
    u64: 8
}

aligned_self! {
    u8,
    i8,
    (),
    bool
}

#[cfg(target_pointer_width = "32")]
mod impl32 {
    use super::Aligned;
    aligned_impl! { isize: 4, usize: 4 }
    unsafe impl<T: Sized> Aligned for *const T { type Unaligned = [u8; 4]; aligned_assert!(Self); }
    unsafe impl<T: Sized> Aligned for *mut T { type Unaligned = [u8; 4]; aligned_assert!(Self); }
    unsafe impl<'a, T: Sized> Aligned for &'a T { type Unaligned = [u8; 4]; }
    unsafe impl<'a, T: Sized> Aligned for &'a mut T { type Unaligned = [u8; 4]; }
}

#[cfg(target_pointer_width = "64")]
mod impl64 {
    use super::Aligned;
    aligned_impl! { isize: 8, usize: 8 }
    unsafe impl<T: Sized> Aligned for *const T { type Unaligned = [u8; 8]; aligned_assert!(Self); }
    unsafe impl<T: Sized> Aligned for *mut T { type Unaligned = [u8; 8]; aligned_assert!(Self); }
    unsafe impl<'a, T: Sized> Aligned for &'a T { type Unaligned = [u8; 8]; }
    unsafe impl<'a, T: Sized> Aligned for &'a mut T { type Unaligned = [u8; 8]; }
}

//unsafe impl<T: Unaligned> Aligned for T { type Unaligned = T; }

unsafe impl Packed for () { }
unsafe impl Packed for i8 { }
unsafe impl Packed for u8 { }
unsafe impl Packed for bool { }
unsafe impl<T> Packed for PhantomData<T> { }

macro_rules! packed_def {
    (=> $($($x:ident),*;)*) => {
        $(
            unsafe impl<$($x: Unaligned),*> Unaligned for ($($x),*,) { }

            // TODO: The language probably doesn't guarantee ordering for tuples, even when all are 1-aligned, so this is incorrect?
            unsafe impl<$($x: Unaligned),*> Packed for ($($x),*,) { }
        )*
    };
    ($($x:expr),*) => {
        $(
            unsafe impl<T: Unaligned> Unaligned for [T; $x] { }

            unsafe impl<T: Unaligned> Packed for [T; $x] { }
        )*
    };
}

unsafe impl<T: Aligned> Aligned for (T,) {
    type Unaligned = T::Unaligned;
}

unsafe impl<T: Aligned> Aligned for [T; 1] {
    type Unaligned = T::Unaligned;
}

unsafe impl<T> Aligned for [T; 0] {
    type Unaligned = ();
}

packed_def! { 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f }
packed_def! { 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f }
packed_def! { 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f }
packed_def! { 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f }
packed_def! { 0x40 }
packed_def! { =>
    A;
    A, B;
    A, B, C;
    A, B, C, D;
    A, B, C, D, E;
    A, B, C, D, E, F;
    A, B, C, D, E, F, G;
    A, B, C, D, E, F, G, H;
    A, B, C, D, E, F, G, H, I;
    A, B, C, D, E, F, G, H, I, J;
    A, B, C, D, E, F, G, H, I, J, K;
}

#[test]
fn assert_packed() {
    fn is<T: Packed>() { }
    fn is_unaligned<T: Unaligned>() { }

    is::<()>();
    is::<u8>();
    is::<i8>();
    is::<bool>();
    is_unaligned::<(bool, u8)>();
}
