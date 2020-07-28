use super::ffi::{AsMutPtr, AsPtr};
use super::Connection;
use glib::translate::FromGlib;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::NonNull;

pub struct ChatConversation(NonNull<purple_sys::PurpleConvChat>);

impl ChatConversation {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleConvChat) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn find(connection: &mut Connection, id: i32) -> Option<Self> {
        unsafe {
            Self::from_ptr(purple_sys::purple_find_chat(connection.as_mut_ptr(), id)
                as *mut purple_sys::PurpleConvChat)
        }
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

    pub fn get_data(&mut self, key: &str) -> Option<&str> {
        unsafe {
            let c_key = CString::new(key).unwrap();
            NonNull::new(purple_sys::purple_conversation_get_data(
                self.as_conversation_ptr(),
                c_key.as_ptr(),
            ))
            .map(|p| {
                CStr::from_ptr(p.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap()
            })
        }
    }

    pub fn set_title(&mut self, title: &str) {
        unsafe {
            let c_title = CString::new(title).unwrap();
            purple_sys::purple_conversation_set_title(self.as_conversation_ptr(), c_title.as_ptr());
        }
    }

    pub fn get_title(&mut self) -> Option<&str> {
        unsafe {
            let c_value = purple_sys::purple_conversation_get_title(self.as_conversation_ptr());
            NonNull::new(c_value as *mut c_char).map(|p| {
                CStr::from_ptr(p.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap()
            })
        }
    }

    pub fn get_connection(&mut self) -> Connection {
        unsafe {
            let c_connection = purple_sys::purple_conversation_get_gc(self.as_conversation_ptr());
            Connection::from_raw(c_connection).unwrap()
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
