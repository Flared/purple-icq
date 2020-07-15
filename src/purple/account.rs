use super::Connection;
use purple_sys;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
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

    pub fn request_input<F>(
        &self,
        title: Option<&str>,
        primary: Option<&str>,
        secondary: Option<&str>,
        default_value: Option<&str>,
        multiline: bool,
        masked: bool,
        hint: Option<&str>,
        ok_text: &str,
        cancel_text: &str,
        callback: F,
        who: Option<&str>,
    ) {
        let title = title.map(|v|CString::new(v).unwrap().into_raw());
        let primary = primary.map(|v|CString::new(v).unwrap().into_raw());
        let ok_text = CString::new(ok_text).unwrap().into_raw();
        let cancel_text = CString::new(cancel_text).unwrap().into_raw();

        let connection = self.get_connection();

        let callback_closure = |value| {};
        let user_data = Box::into_raw(Box::new(callback_closure)) as *mut c_void;

        purple_sys::purple_request_input(
            connection,
            title,
            primary,
            primary,
            primary,
            multiline,
            masked,
            primary,
            ok_text,
            request_input_ok_trampoline,
            cancel_text,
            request_input_cancel_trampoline,
            self.0,
            primary,
            std::ptr::null_mut(),
            user_data
    }
}

unsafe extern "C" fn request_input_ok_trampoline<F>(user_data: *mut c_void, value: *const c_char)
where
    F: FnOnce(Option<&CStr>),
{
    if let Err(error) = catch_unwind(|| {
        let value = CStr::from_ptr(value).unwrap();
        let closure = &*(user_data as *const F);
        closure(Ok(value.to_string_lossy()));
    }) {
        log::error!("Error in request_input callback: {:?}", error);
    }
}

unsafe extern "C" fn request_input_cancel_trampoline<F>(user_data: *mut c_void)
where
    F: FnOnce(Option<&CStr>),
{
    let closure = &*(user_data as *const F);
    closure(None);
}
