use std::os::raw::c_void;

pub struct GList {
    ptr: *mut glib_sys::GList,
}

impl GList {
    pub fn new() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn append(&mut self, value: *mut c_void) {
        unsafe {
            self.ptr = glib_sys::g_list_append(self.ptr, value);
        }
    }

    pub fn into_raw(self) -> *mut glib_sys::GList {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}

impl From<Vec<*mut c_void>> for GList {
    fn from(vec: Vec<*mut c_void>) -> Self {
        let mut list = Self::new();
        for item in vec {
            list.append(item)
        }
        list
    }
}
