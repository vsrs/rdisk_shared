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

pub trait AsByteSlice {
    /// # Safety
    /// The method is unsafe because any padding bytes in the struct may be uninitialized memory (giving undefined behavior).
    /// Also, there are not any Endianness assumtions. The caller should care about it.
    unsafe fn as_byte_slice(&self) -> &[u8];
}

pub trait AsByteSliceMut {
    /// # Safety
    /// The method is unsafe because any padding bytes in the struct may be uninitialized memory (giving undefined behavior).
    /// Also, there are not any Endianness assumtions. The caller should care about it.
    unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8];
}

macro_rules! impl_int {
    ($name:ty) => {
        impl AsByteSlice for $name {
            unsafe fn as_byte_slice(&self) -> &[u8] {
                let byte_size = core::mem::size_of::<$name>();
                core::slice::from_raw_parts(self as *const _ as *const u8, byte_size)
            }
        }

        impl AsByteSlice for [$name] {
            unsafe fn as_byte_slice(&self) -> &[u8] {
                let byte_size = self.len() * core::mem::size_of::<$name>();
                core::slice::from_raw_parts(self.as_ptr() as *const u8, byte_size)
            }
        }
        
        impl AsByteSlice for Vec<$name> {
            unsafe fn as_byte_slice(&self) -> &[u8] {
                let byte_size = self.len() * core::mem::size_of::<$name>();
                core::slice::from_raw_parts(self.as_ptr() as *const u8, byte_size)
            }
        }

        impl AsByteSliceMut for $name {
            unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8] {
                let byte_size = core::mem::size_of::<$name>();
                core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, byte_size)
            }
        }

        impl AsByteSliceMut for [$name] {
            unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8] {
                let byte_size = self.len() * core::mem::size_of::<$name>();
                core::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, byte_size)
            }
        }
        
        impl AsByteSliceMut for Vec<$name> {
            unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8] {
                let byte_size = self.len() * core::mem::size_of::<$name>();
                core::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, byte_size)
            }
        }
    };
}

impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);

pub struct StructBuffer<T: Sized> {
    buffer: Vec<u8>,
    _marker: core::marker::PhantomData<T>
}

#[allow(clippy::len_without_is_empty)]
impl<T: Sized + Clone + Copy> StructBuffer<T> {
    /// Creates a buffer capable to hold the value of type `T`.
    /// 
    /// # Safety
    /// The buffer is uninitialized! 
    pub unsafe fn new() -> Self {
        Self{
            buffer: alloc_buffer(core::mem::size_of::<T>()),
            _marker: Default::default()
        }
    }

    /// Creates a buffer capable to hold the value of type `T` plus `ext_size` bytes.
    /// 
    /// # Safety
    /// The buffer is uninitialized! 
    pub unsafe fn with_ext(ext_size: usize) -> Self {
        Self{
            buffer: alloc_buffer(core::mem::size_of::<T>() + ext_size),
            _marker: Default::default()
        }
    }

    /// Creates a StructBuffer for the type `T` using supplied `buffer`.
    /// 
    /// # Safety
    /// The buffer size should be >= mem::size_of::<T>() ! 
    pub unsafe fn with_buffer(buffer: Vec<u8>) -> Self {
        if buffer.len() < core::mem::size_of::<T>() {
            panic!("Insufficient buffer size!")
        }

        Self{
            buffer,
            _marker: Default::default()
        }
    }

    /// Creates the value of type `T` represented by the all-zero byte-pattern.
    pub fn zeroed() -> Self {
        Self{
            buffer: vec![0_u8; core::mem::size_of::<T>()],
            _marker: Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn raw(&self) -> &T {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe { &*(self.buffer.as_ptr() as *const T) }
    }

    pub fn raw_mut(&mut self) -> &mut T {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe { &mut *(self.buffer.as_mut_ptr() as *mut T) }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn ext_buffer(&self) -> &[u8] {
        &self.buffer[core::mem::size_of::<T>()..]
    }

    pub fn ext_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[core::mem::size_of::<T>()..]
    }

    pub fn has_ext_buffer(&self) -> bool {
        !self.ext_buffer().is_empty()
    }

    pub fn copy(&self) -> T {
        *self.raw()
    }

    pub fn take(self) -> T {
        *self.raw()
    }
}

impl<T:Sized + Clone + Copy> core::ops::Deref for StructBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.raw()
    }
}

impl<T:Sized + Clone + Copy> core::ops::DerefMut for StructBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw_mut()
    }
}

impl<T:Sized + Clone + Copy> AsByteSlice for StructBuffer<T> {
    unsafe fn as_byte_slice(&self) -> &[u8] {
        self.buffer.as_byte_slice()
    }
}

impl<T:Sized + Clone + Copy> AsByteSliceMut for StructBuffer<T> {
    unsafe fn as_byte_slice_mut(&mut self) -> &mut [u8] {
        self.buffer.as_byte_slice_mut()
    }
}

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

    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct S {
        byte: u8,
        word: u16
    }

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
        let mut buffer = StructBuffer::<S>::zeroed();
        assert_eq!(3, buffer.len());

        unsafe {
            // packed fileds
            assert_eq!(0, buffer.byte);
            assert_eq!(0, buffer.word )
        }

        buffer.byte = 12;
        unsafe {
            assert_eq!(12, buffer.byte);
            assert_eq!(0, buffer.word )
        }

        let bytes = unsafe { buffer.as_byte_slice() };
        assert_eq!(3, bytes.len());

        let s = buffer.copy();
        unsafe {
            assert_eq!(12, s.byte);
            assert_eq!(0, s.word )
        }

        let s = buffer.take();
        unsafe {
            assert_eq!(12, s.byte);
            assert_eq!(0, s.word )
        }
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

    #[test]
    fn ext_buffer() {
        let mut buffer = unsafe { StructBuffer::<S>::with_ext(4) };
        assert_eq!(7, buffer.len());
        assert!(buffer.has_ext_buffer());
        assert!(buffer.ext_buffer().len() == 4);
        assert!(buffer.ext_buffer_mut().len() == 4);

        let mut buffer = StructBuffer::<S>::zeroed();
        assert_eq!(3, buffer.len());
        assert!(!buffer.has_ext_buffer());
        assert!(buffer.ext_buffer().len() == 0);
        assert!(buffer.ext_buffer_mut().len() == 0);
    }
}