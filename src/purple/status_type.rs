use super::ffi::{mut_override, AsPtr};
use glib::translate::ToGlib;
pub use purple_sys::PurpleStatusPrimitive;
use std::ffi::CStr;

pub struct StatusType(*mut purple_sys::PurpleStatusType);

impl StatusType {
    pub fn new(
        primitive: PurpleStatusPrimitive,
        id: Option<&'static CStr>,
        name: Option<&'static CStr>,
        user_settable: bool,
    ) -> Self {
        unsafe {
            Self(purple_sys::purple_status_type_new(
                primitive,
                id.map_or_else(std::ptr::null_mut, |s| mut_override(s.as_ptr())),
                name.map_or_else(std::ptr::null_mut, |s| mut_override(s.as_ptr())),
                user_settable.to_glib(),
            ))
        }
    }
}

impl AsPtr for StatusType {
    type PtrType = purple_sys::PurpleStatusType;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0
    }
}
