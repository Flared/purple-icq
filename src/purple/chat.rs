use super::ffi::{AsMutPtr, AsPtr, IntoGlibPtr};
use crate::purple;
use std::ffi::CString;
use std::ptr::{null_mut, NonNull};

pub struct Chat(NonNull<purple_sys::PurpleChat>);

impl Chat {
    pub fn new(
        account: &mut purple::Account,
        alias: &str,
        components: purple::StrHashTable,
    ) -> Self {
        let c_alias = CString::new(alias).unwrap();
        unsafe {
            Self::from_ptr(purple_sys::purple_chat_new(
                account.as_mut_ptr(),
                c_alias.as_ptr(),
                components.into_glib_full(),
            ))
            .unwrap()
        }
    }

    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleChat) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn find(account: &mut purple::Account, name: &str) -> Option<Self> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            Self::from_ptr(purple_sys::purple_blist_find_chat(
                account.as_mut_ptr(),
                c_name.as_ptr(),
            ))
        }
    }

    pub fn add_to_blist(&mut self, group: &mut purple::Group, _node: Option<()>) {
        unsafe {
            purple_sys::purple_blist_add_chat(self.as_mut_ptr(), group.as_mut_ptr(), null_mut())
        }
    }
}

impl AsPtr for Chat {
    type PtrType = purple_sys::PurpleChat;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
