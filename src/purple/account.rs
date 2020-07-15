use super::ffi::{AsMutPtr, AsPtr};
use super::Connection;
use purple_sys;
use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::panic::catch_unwind;
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
            unsafe { Connection::from_raw(connection_ptr) }
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
        _callback: F,
        who: Option<&str>,
    ) {
        let title = title.map(|v| CString::new(v).unwrap().into_raw());
        let primary = primary.map(|v| CString::new(v).unwrap().into_raw());
        let secondary = secondary.map(|v| CString::new(v).unwrap().into_raw());
        let default_value = default_value.map(|v| CString::new(v).unwrap().into_raw());
        let mut hint = hint.map(|v| CString::new(v).unwrap().into_raw());
        let who = who.map(|v| CString::new(v).unwrap().into_raw());
        let ok_text = CString::new(ok_text).unwrap().into_raw();
        let cancel_text = CString::new(cancel_text).unwrap().into_raw();

        let mut connection = self.get_connection().map(|mut c| c.as_ptr());

        let callback_closure = |value: Option<Cow<str>>| {
            log::info!("In closure");
            log::info!("value: {}", value.unwrap_or("<cancel>".into()));
        };
        let user_data = Box::into_raw(Box::new(callback_closure)) as *mut c_void;

        let request_input_ok_trampoline_ptr: RequestInputOkTrampoline = request_input_ok_trampoline;

        let request_input_cancel_trampoline_ptr: RequestInputCancelTrampoline =
            request_input_cancel_trampoline;

        unsafe {
            purple_sys::purple_request_input(
                connection.as_mut_ptr() as *mut c_void,
                title.as_ptr(),
                primary.as_ptr(),
                secondary.as_ptr(),
                default_value.as_ptr(),
                multiline as i32,
                masked as i32,
                hint.as_mut_ptr(),
                ok_text,
                None,
                cancel_text,
                None
                self.0,
                who.as_ptr(),
                std::ptr::null_mut(),
                user_data,
            );
        }
    }
}

type RequestInputCallback = dyn FnOnce(Option<Cow<str>>) + 'static + std::panic::RefUnwindSafe;
struct RequestInputData {
    callback: Box<RequestInputCallback>,
}

type RequestInputOkTrampoline = unsafe extern "C" fn(*mut c_void, *const i8);
type RequestInputCancelTrampoline = unsafe extern "C" fn(*mut c_void);

unsafe extern "C" fn request_input_ok_trampoline::<F>(user_data: *mut c_void, value: *const c_char) {
    let user_data: *mut RequestInputCallback = std::mem::transmute(user_data);
    log::debug!("request_input_ok_trampoline");
    if let Err(error) = catch_unwind(|| {
        let value = CStr::from_ptr(value);
        let closure: Box<dyn FnOnce(Option<Cow<str>>) + 'static + std::panic::RefUnwindSafe> =
            Box::from_raw(user_data);
        closure(Some(value.to_string_lossy()));
    }) {
        log::error!("Error in request_input callback: {:?}", error);
    }
}

unsafe extern "C" fn request_input_cancel_trampoline(user_data: *mut c_void) {
    let user_data: *mut RequestInputCallback = std::mem::transmute(user_data);
    log::debug!("request_input_cancel_trampoline");
    if let Err(error) = catch_unwind(|| {
        let closure: Box<dyn FnOnce(Option<Cow<str>>) + 'static + std::panic::RefUnwindSafe> =
            Box::from_raw(user_data);
        log::info!("Calling closure");
        closure(None);
    }) {
        log::error!("Error in request_input callback: {:?}", error);
    }
}



fn get_trampoline_data<F>(f: F) -> *mut c_void
where F: FnOnce() {
    Box::into_raw(Box::new(callback)) as *mut c_void
}

unsafe extern "C" fn trampoline<F>(user_data: *mut c_void)
where
    F: FnOnce()
{
    let closure = &*(user_data as *mut F);
    closure();
}
