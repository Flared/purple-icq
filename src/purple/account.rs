use super::Connection;
use purple_sys;
use std::borrow::Cow;
use std::ffi::CStr;
pub struct Account(*mut purple_sys::PurpleAccount);

impl Account {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleAccount) -> Self {
        Account(ptr)
    }

    pub fn as_ptr(&self) -> *mut purple_sys::PurpleAccount {
        self.0
    }

    pub fn get_connection(&self) -> Option<Connection> {
        let connection_ptr = unsafe { purple_sys::purple_account_get_connection(self.0) };
        if connection_ptr.is_null() {
            None
        } else {
            Some(unsafe { Connection::from_raw(connection_ptr) })
        }
    }

    pub fn get_username(&self) -> Option<Cow<str>> {
        let username_ptr = unsafe { purple_sys::purple_account_get_username(self.0) };
        if username_ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(username_ptr) }.to_string_lossy())
        }
    }
}
