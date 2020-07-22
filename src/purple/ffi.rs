use glib::translate::{Ptr, ToGlibPtr};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};

pub fn mut_override<T>(ptr: *const T) -> *mut T {
    ptr as *mut T
}
pub trait IntoGlibPtr<P> {
    fn into_glib_full(self) -> *mut P;
}

impl<P: AsMutPtr> IntoGlibPtr<P::PtrType> for P {
    fn into_glib_full(mut self) -> *mut P::PtrType {
        let ptr = self.as_mut_ptr();
        std::mem::forget(self);
        ptr
    }
}

pub trait ToGlibContainerFromIterator<P>
where
    Self: Sized,
{
    fn into_glib_full_from_iter<I: IntoIterator<Item = Self>>(iter: I) -> P;
}

impl<T, P> ToGlibContainerFromIterator<*mut glib_sys::GList> for T
where
    T: AsPtr<PtrType = P>,
    T: IntoGlibPtr<P>,
    P: 'static,
{
    fn into_glib_full_from_iter<I>(iter: I) -> *mut glib_sys::GList
    where
        I: IntoIterator<Item = Self>,
    {
        let mut list: *mut glib_sys::GList = std::ptr::null_mut();
        unsafe {
            for ptr in iter.into_iter().map(|v| v.into_glib_full()) {
                list = glib_sys::g_list_append(list, Ptr::to(ptr));
            }
        }
        list
    }
}

impl ToGlibContainerFromIterator<*mut glib_sys::GHashTable> for (&'static CStr, String) {
    fn into_glib_full_from_iter<I>(iter: I) -> *mut glib_sys::GHashTable
    where
        I: IntoIterator<Item = Self>,
    {
        unsafe {
            let ptr = glib_sys::g_hash_table_new_full(
                Some(glib_sys::g_str_hash),
                Some(glib_sys::g_str_equal),
                None,
                Some(glib_sys::g_free),
            );
            for (k, v) in iter {
                let k: *const c_char = k.as_ptr();
                let v: *mut c_char = v.to_glib_full();
                glib_sys::g_hash_table_insert(ptr, k as *mut _, v as *mut _);
            }
            ptr
        }
    }
}

pub trait AsPtr {
    type PtrType;
    fn as_ptr(&self) -> *const Self::PtrType;
}

pub trait AsMutPtr {
    type PtrType;
    fn as_mut_ptr(&mut self) -> *mut Self::PtrType;
}

impl<T: AsPtr> AsMutPtr for T {
    type PtrType = T::PtrType;
    fn as_mut_ptr(&mut self) -> *mut Self::PtrType {
        self.as_ptr() as *mut Self::PtrType
    }
}

impl<T> AsPtr for Option<NonNull<T>> {
    type PtrType = T;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.map_or_else(null, |x| x.as_ptr())
    }
}

impl<T> AsPtr for Option<*const T> {
    type PtrType = T;
    fn as_ptr(&self) -> *const T {
        self.unwrap_or_else(null)
    }
}

impl<T> AsPtr for Option<*mut T> {
    type PtrType = T;
    fn as_ptr(&self) -> *const T {
        self.unwrap_or_else(null_mut)
    }
}
