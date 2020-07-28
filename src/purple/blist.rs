use super::ffi::AsPtr;
use libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::ptr::NonNull;

pub struct BlistNode(NonNull<purple_sys::PurpleBlistNode>);

impl BlistNode {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleBlistNode) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn get_string(&mut self, key: &str) -> Option<&str> {
        let c_key = CString::new(key).unwrap();

        unsafe {
            let c_value = purple_sys::purple_blist_node_get_string(self.0.as_ptr(), c_key.as_ptr());
            NonNull::new(c_value as *mut c_char).map(|p| {
                CStr::from_ptr(p.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap()
            })
        }
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        let c_key = CString::new(key).unwrap();
        let c_value = CString::new(value).unwrap();
        unsafe {
            purple_sys::purple_blist_node_set_string(
                self.0.as_ptr(),
                c_key.as_ptr(),
                c_value.as_ptr(),
            );
        }
    }
}

impl AsPtr for BlistNode {
    type PtrType = purple_sys::PurpleBlistNode;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
