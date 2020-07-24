use super::ffi::{mut_override, AsPtr};
use glib::translate::{FromGlib, FromGlibPtrContainer, ToGlibPtr};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::NonNull;

pub type StrHashTable<'a> = HashTable<&'static CStr, &'a str>;

pub struct HashTable<K, V>(
    NonNull<glib_sys::GHashTable>,
    std::marker::PhantomData<(K, V)>,
);

impl Default for HashTable<&'static CStr, &str> {
    fn default() -> Self {
        Self(
            NonNull::new(unsafe {
                glib_sys::g_hash_table_new_full(
                    Some(glib_sys::g_str_hash),
                    Some(glib_sys::g_str_equal),
                    None,
                    Some(glib_sys::g_free),
                )
            })
            .unwrap(),
            std::marker::PhantomData,
        )
    }
}

impl HashTable<&'static CStr, &str> {
    pub unsafe fn from_ptr(ptr: *mut glib_sys::GHashTable) -> Option<Self> {
        NonNull::new(ptr).map(|p| Self(p, std::marker::PhantomData))
    }

    pub fn insert(&mut self, key: &'static CStr, value: &str) -> bool {
        FromGlib::from_glib(unsafe {
            glib_sys::g_hash_table_insert(
                self.0.as_ptr(),
                key.as_ptr() as *mut c_void,
                ToGlibPtr::<*mut c_char>::to_glib_full(value) as *mut c_void,
            )
        })
    }

    pub fn lookup(&self, key: &'static CStr) -> Option<&str> {
        unsafe {
            NonNull::new(glib_sys::g_hash_table_lookup(
                mut_override(self.as_ptr()),
                key.as_ptr() as *const c_void,
            ) as *mut c_char)
            .map(|p| CStr::from_ptr(p.as_ptr()).to_str().unwrap())
        }
    }
}

impl<K, V> std::fmt::Debug for HashTable<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::collections::HashMap;
        let hashmap: HashMap<String, String> =
            unsafe { FromGlibPtrContainer::from_glib_none(self.0.as_ptr()) };
        write!(f, "HashTable({:?})", hashmap)
    }
}

impl<K, V> AsPtr for HashTable<K, V> {
    type PtrType = glib_sys::GHashTable;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
