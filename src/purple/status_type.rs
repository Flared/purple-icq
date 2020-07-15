pub use purple_sys::PurpleStatusPrimitive;
use std::ffi::CString;
pub struct StatusType {
    ptr: *mut purple_sys::PurpleStatusType,
}

impl StatusType {
    pub fn new(
        primitive: PurpleStatusPrimitive,
        id: String,
        name: String,
        user_settable: bool,
    ) -> Self {
        let status_type_ptr = unsafe {
            purple_sys::purple_status_type_new(
                primitive,
                CString::new(id).unwrap().into_raw(),
                CString::new(name).unwrap().into_raw(),
                user_settable as i32,
            )
        };
        Self {
            ptr: status_type_ptr,
        }
    }

    pub fn into_raw(self) -> *mut purple_sys::PurpleStatusType {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }
}
