use std::ptr::{null, null_mut, NonNull};

pub trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
}

pub trait AsMutPtr<T> {
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl<T> AsPtr<T> for Option<NonNull<T>> {
    fn as_ptr(&self) -> *const T {
        self.map_or_else(null, |x| x.as_ptr())
    }
}

impl<T> AsMutPtr<T> for Option<NonNull<T>> {
    fn as_mut_ptr(&mut self) -> *mut T {
        self.map_or_else(null_mut, |x| x.as_ptr())
    }
}

impl<T> AsPtr<T> for Option<*const T> {
    fn as_ptr(&self) -> *const T {
        self.unwrap_or_else(null)
    }
}

impl<T> AsPtr<T> for Option<*mut T> {
    fn as_ptr(&self) -> *const T {
        self.unwrap_or_else(null_mut)
    }
}

impl<T> AsMutPtr<T> for Option<*mut T> {
    fn as_mut_ptr(&mut self) -> *mut T {
        self.unwrap_or_else(null_mut)
    }
}
