use super::ffi::{AsMutPtr, AsPtr};
use std::ffi::CString;
use std::ptr::{null_mut, NonNull};

pub struct Group(NonNull<purple_sys::PurpleGroup>);

impl Group {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleGroup) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn find(name: &str) -> Option<Group> {
        let c_name = CString::new(name).unwrap();
        unsafe { Group::from_ptr(purple_sys::purple_find_group(c_name.as_ptr())) }
    }

    pub fn new(name: &str) -> Group {
        let c_name = CString::new(name).unwrap();
        unsafe { Group::from_ptr(purple_sys::purple_group_new(c_name.as_ptr())).unwrap() }
    }

    pub fn add_to_blist(&mut self, _node: Option<()>) {
        unsafe { purple_sys::purple_blist_add_group(self.as_mut_ptr(), null_mut()) }
    }
}

impl AsPtr for Group {
    type PtrType = purple_sys::PurpleGroup;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
