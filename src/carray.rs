use std::borrow::Cow;
use std::ffi::{c_char, CStr};

pub fn pchar_to_cowstr<'a>(p: *const c_char) -> Cow<'a, str> {
    unsafe { CStr::from_ptr(p).to_string_lossy() }
}


/// Iterates through a C array of any type [T] described by pointer and element count.
pub struct CArrayIterator<T> {
    /// Pointer to the array
    ptr: *const T,
    /// Number of remaining elements
    count: usize,
}

impl<T> CArrayIterator<T> {
    pub(crate) fn new(ptr: *const T, count: usize) -> impl Iterator<Item=T> {
        Self { ptr, count }
    }
}

impl<T> Iterator for CArrayIterator<T>{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            None
        } else {
            self.count -= 1;
            // SAFETY: The caller provides pointer to array of T and its correct size.
            // Therefore we just have to stop before reaching the end.
            unsafe {
                let value = self.ptr.read();
                self.ptr = self.ptr.add(1);
                Some(value)
            }
        }
    }
}

/// Iterates through two C arrays of any types [T1],[T2] described by pointers and element count.
/// The count must be equal for both these arrays.
pub struct CTwinArrayIterator<T1, T2> {
    /// Pointer to the first array
    ptr1: *const T1,
    ptr2: *const T2,
    /// Number of remaining elements
    count: usize,
}

impl<T1, T2> CTwinArrayIterator<T1, T2> {
    pub(crate) fn new(count: usize, ptr1: *const T1, ptr2: *const T2) -> impl Iterator<Item=(T1, T2)> {
        CTwinArrayIterator { ptr1, ptr2, count, }
    }
}

impl<T1,T2> Iterator for CTwinArrayIterator<T1,T2>{
    type Item = (T1, T2);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            None
        } else {
            self.count -= 1;
            // SAFETY: The caller provides pointers to arrays and their correct size.
            // Therefore we just have to stop before reaching the end.
            unsafe {
                let value1 = self.ptr1.read();
                let value2 = self.ptr2.read();
                self.ptr1 = self.ptr1.add(1);
                self.ptr2 = self.ptr2.add(1);
                Some((value1,value2))
            }
        }
    }
}
