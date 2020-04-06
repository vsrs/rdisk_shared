#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc as std;

pub use std::collections::BTreeMap;
pub use std::string::String;
pub use std::vec::Vec;

pub use core::option::Option;

pub trait NullSafePtr<T: Sized> {
    fn safe_ptr(&self) -> *const T;
}

pub trait NullSafeMutPtr<T: Sized> {
    fn safe_mut_ptr(&mut self) -> *mut T;
}

impl<T: Sized> NullSafePtr<T> for &[T] {
    fn safe_ptr(&self) -> *const T {
        if self.is_empty() {
            core::ptr::null()
        } else {
            self.as_ptr()
        }
    }
}

impl<T: Sized> NullSafeMutPtr<T> for &mut [T] {
    fn safe_mut_ptr(&mut self) -> *mut T {
        if self.is_empty() {
            core::ptr::null_mut()
        } else {
            self.as_mut_ptr()
        }
    }
}

impl<T: Sized> NullSafePtr<T> for Vec<T> {
    fn safe_ptr(&self) -> *const T {
        self.as_slice().safe_ptr()
    }
}

impl NullSafePtr<u8> for str {
    fn safe_ptr(&self) -> *const u8 {
        self.as_bytes().safe_ptr()
    }
}

/// A type is Byteable if it might be represented as a byte sequence.
/// 
/// Any struct that implements Byteable automatically gets AsByteSlice and AsByteSliceMut traits implementations.
/// 
/// # Safety
/// The trait is unsafe because any padding bytes in the struct may be uninitialized memory (giving undefined behavior).
/// Also, there are not any Endianness assumtions. The caller should care about it.
pub unsafe trait Byteable : Sized {}

unsafe impl Byteable for u8 {}
unsafe impl Byteable for u16 {}
unsafe impl Byteable for u32 {}
unsafe impl Byteable for u64 {}
unsafe impl Byteable for i8 {}
unsafe impl Byteable for i16 {}
unsafe impl Byteable for i32 {}
unsafe impl Byteable for i64 {}

/// Companion trait for Byteable.
pub unsafe trait AsByteSlice {
    /// # Safety
    /// There are not any Endianness assumtions!
    unsafe fn as_byte_slice(&self) -> &[u8];
}

/// Companion trait for Byteable.
pub unsafe trait AsByteSliceMut {
    /// # Safety
    /// There are not any Endianness assumtions!
    unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8];
}

unsafe impl<T: Byteable> AsByteSlice for T {
    unsafe fn as_byte_slice(&self) -> &[u8] {
        core::slice::from_raw_parts((self as *const T) as *const u8, core::mem::size_of::<T>())
    }
}

unsafe impl<T: Byteable> AsByteSliceMut for T {
    unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        core::slice::from_raw_parts_mut((self as *mut T) as *mut u8, core::mem::size_of::<T>())
    }
}

unsafe impl<T: Byteable> AsByteSlice for &[T] {
    unsafe fn as_byte_slice(&self) -> &[u8] {
        let byte_size = self.len() * core::mem::size_of::<T>();
        core::slice::from_raw_parts(self.as_ptr() as *const u8, byte_size)
    }
}

unsafe impl<T: Byteable> AsByteSlice for Vec<T> {
    unsafe fn as_byte_slice(&self) -> &[u8] {
        let byte_size = self.len() * core::mem::size_of::<T>();
        core::slice::from_raw_parts(self.as_ptr() as *const u8, byte_size)
    }
}

#[allow(dead_code)]
/// # Safety
/// The allocated buffer is uninitialized and should be entirely rewritten before read.
pub unsafe fn alloc_buffer(size: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(size);
    buffer.set_len(size);
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_byte_slice_for_vec() {
        let vec: Vec<u8> = vec![1, 2, 3];
        let bytes = unsafe { vec.as_byte_slice() };
        assert_eq!(3, bytes.len());

        let vec: Vec<u16> = vec![1, 2, 3];
        let bytes = unsafe { vec.as_byte_slice() };
        assert_eq!(6, bytes.len());

        let vec: Vec<u32> = vec![1, 2, 3];
        let bytes = unsafe { vec.as_byte_slice() };
        assert_eq!(12, bytes.len());
    }

    #[test]
    fn as_byte_slice_for_slice() {
        let vec: Vec<u8> = vec![1, 2, 3];
        let slice = vec.as_slice();
        let bytes = unsafe { slice.as_byte_slice() };
        assert_eq!(3, bytes.len());

        let vec: Vec<u16> = vec![1, 2, 3];
        let slice = vec.as_slice();
        let bytes = unsafe { slice.as_byte_slice() };
        assert_eq!(6, bytes.len());

        let vec: Vec<u32> = vec![1, 2, 3];
        let slice = vec.as_slice();
        let bytes = unsafe { slice.as_byte_slice() };
        assert_eq!(12, bytes.len());
    }

    #[test]
    fn as_byte_slice_for_struct() {

        #[repr(C, packed)]
        struct S {
            byte: u8,
            word: u16
        };

        unsafe impl Byteable for S{};

        let s = S{byte:1, word: 3};

        let bytes = unsafe { s.as_byte_slice() };
        assert_eq!(3, bytes.len());
    }

    #[test]
    fn as_byte_slice_for_primitive() {
        let b = 4_u8;
        let bytes = unsafe { b.as_byte_slice() };
        assert_eq!(1, bytes.len());

        let b = 4_u16;
        let bytes = unsafe { b.as_byte_slice() };
        assert_eq!(2, bytes.len());

        let b = 4_u32;
        let bytes = unsafe { b.as_byte_slice() };
        assert_eq!(4, bytes.len());

        let b = 4_u64;
        let bytes = unsafe { b.as_byte_slice() };
        assert_eq!(8, bytes.len());
    }
}