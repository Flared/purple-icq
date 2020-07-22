use super::ffi::AsPtr;
use glib::translate::FromGlib;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;

pub struct ChatConversation(NonNull<purple_sys::PurpleConvChat>);

impl ChatConversation {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleConvChat) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn has_left(&mut self) -> bool {
        FromGlib::from_glib(unsafe { purple_sys::purple_conv_chat_has_left(self.0.as_ptr()) })
    }

    pub fn present(&mut self) {
        unsafe { purple_sys::purple_conversation_present(self.as_conversation_ptr()) }
    }

    pub fn set_data(&mut self, key: &str, data: &str) {
        unsafe {
            let c_key = CString::new(key).unwrap();
            let c_data = CString::new(data).unwrap();
            purple_sys::purple_conversation_set_data(
                self.as_conversation_ptr(),
                c_key.as_ptr(),
                c_data.into_raw() as *mut c_void,
            );
        }
    }

    pub fn set_title(&mut self, title: &str) {
        unsafe {
            let c_title = CString::new(title).unwrap();
            purple_sys::purple_conversation_set_title(self.as_conversation_ptr(), c_title.as_ptr());
        }
    }

    pub fn as_conversation_ptr(&mut self) -> *mut purple_sys::PurpleConversation {
        self.0.as_ptr() as *mut purple_sys::PurpleConversation
    }
}

impl AsPtr for ChatConversation {
    type PtrType = purple_sys::PurpleConvChat;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
