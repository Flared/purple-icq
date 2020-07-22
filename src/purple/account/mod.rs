use super::ffi::{AsMutPtr, AsPtr};
use super::Connection;
use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::panic::catch_unwind;

pub mod settings;

impl AsMutPtr for Account {
    type PtrType = purple_sys::PurpleAccount;
    fn as_mut_ptr(&mut self) -> *mut purple_sys::PurpleAccount {
        self.0
    }
}

pub struct Account(*mut purple_sys::PurpleAccount);

impl Account {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleAccount) -> Self {
        Account(ptr)
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

    #[allow(dead_code)]
    pub fn get_bool(&self, key: &str, default_value: bool) -> bool {
        let c_key = CString::new(key).unwrap();
        unsafe {
            purple_sys::purple_account_get_bool(self.0, c_key.as_ptr(), default_value as i32) != 0
        }
    }
    pub fn get_int(&self, key: &str, default_value: i32) -> i32 {
        let c_key = CString::new(key).unwrap();
        unsafe { purple_sys::purple_account_get_int(self.0, c_key.as_ptr(), default_value) }
    }
    pub fn get_string(&self, key: &str, default_value: &str) -> String {
        let c_key = CString::new(key).unwrap();
        let c_default_value = CString::new(default_value).unwrap();
        let c_value = unsafe {
            purple_sys::purple_account_get_string(self.0, c_key.as_ptr(), c_default_value.as_ptr())
        };
        unsafe { CStr::from_ptr(c_value).to_string_lossy().into_owned() }
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        log::info!("Set setting: {} = {}", key, value);
        let c_key = CString::new(key).unwrap();
        unsafe { purple_sys::purple_account_set_bool(self.0, c_key.as_ptr(), value as i32) };
    }
    pub fn set_int(&self, key: &str, value: i32) {
        log::info!("Set setting: {} = {}", key, value);
        let c_key = CString::new(key).unwrap();
        unsafe { purple_sys::purple_account_set_int(self.0, c_key.as_ptr(), value) };
    }
    pub fn set_string(&self, key: &str, value: &str) {
        log::info!("Set setting: {} = {}", key, value);
        let c_key = CString::new(key).unwrap();
        let c_value = CString::new(value).unwrap();
        unsafe { purple_sys::purple_account_set_string(self.0, c_key.as_ptr(), c_value.as_ptr()) };
    }

    pub fn remove_setting(&self, key: &str) {
        log::info!("Delete setting: {}", key);
        let c_key = CString::new(key).unwrap();
        unsafe { purple_sys::purple_account_remove_setting(self.0, c_key.as_ptr()) };
    }

    pub fn set_settings<T: serde::Serialize>(&self, settings: &T) -> settings::Result<()> {
        settings::to_account(&self, settings)
    }

    #[allow(clippy::too_many_arguments)]
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
    ) where
        F: FnOnce(Option<Cow<str>>) + 'static,
    {
        let title = title.map(|v| CString::new(v).unwrap().into_raw());
        let primary = primary.map(|v| CString::new(v).unwrap().into_raw());
        let secondary = secondary.map(|v| CString::new(v).unwrap().into_raw());
        let default_value = default_value.map(|v| CString::new(v).unwrap().into_raw());
        let mut hint = hint.map(|v| CString::new(v).unwrap().into_raw());
        let who = who.map(|v| CString::new(v).unwrap().into_raw());
        let ok_text = CString::new(ok_text).unwrap().into_raw();
        let cancel_text = CString::new(cancel_text).unwrap().into_raw();

        let mut connection = self.get_connection().map(|mut c| c.as_ptr());

        let callback_closure = move |value: Option<Cow<str>>| {
            // Regain ownership over strings to free them.
            // Safe since all of those pointer where generated from into_raw()
            unsafe {
                title.map(|p| CString::from_raw(p));
                primary.map(|p| CString::from_raw(p));
                secondary.map(|p| CString::from_raw(p));
                default_value.map(|p| CString::from_raw(p));
                hint.map(|p| CString::from_raw(p));
                who.map(|p| CString::from_raw(p));
                CString::from_raw(ok_text);
                CString::from_raw(cancel_text);
                who.map(|p| CString::from_raw(p));
            }

            callback(value);
        };

        unsafe {
            purple_request_input_with_callback(
                connection.as_mut_ptr() as *mut c_void,
                title.as_ptr(),
                primary.as_ptr(),
                secondary.as_ptr(),
                default_value.as_ptr(),
                multiline as i32,
                masked as i32,
                hint.as_mut_ptr(),
                ok_text,
                cancel_text,
                callback_closure,
                self.0,
                who.as_ptr(),
                std::ptr::null_mut(),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn purple_request_input_with_callback<F>(
    connection: *mut c_void,
    title: *const c_char,
    primary: *const c_char,
    secondary: *const c_char,
    default_value: *const c_char,
    multiline: i32,
    masked: i32,
    hint: *mut c_char,
    ok_text: *const c_char,
    cancel_text: *const c_char,
    callback: F,
    account: *mut purple_sys::PurpleAccount,
    who: *const c_char,
    conv: *mut purple_sys::PurpleConversation,
) where
    F: FnOnce(Option<Cow<str>>) + 'static,
{
    let user_data = Box::into_raw(Box::new(callback)) as *mut c_void;
    let ok_cb_ptr: unsafe extern "C" fn(*mut c_void, *const c_char) =
        request_input_ok_trampoline::<F>;
    let cancel_cb_ptr: unsafe extern "C" fn(*mut c_void) = request_input_cancel_trampoline::<F>;

    purple_sys::purple_request_input(
        connection,
        title,
        primary,
        secondary,
        default_value,
        multiline,
        masked,
        hint,
        ok_text,
        Some(std::mem::transmute(ok_cb_ptr)),
        cancel_text,
        Some(std::mem::transmute(cancel_cb_ptr)),
        account,
        who,
        conv,
        user_data,
    );
}

unsafe extern "C" fn request_input_ok_trampoline<F>(user_data: *mut c_void, value: *const c_char)
where
    F: FnOnce(Option<Cow<str>>),
{
    log::debug!("request_input_ok_trampoline");
    if let Err(error) = catch_unwind(|| {
        let value = CStr::from_ptr(value);
        let closure = Box::from_raw(user_data as *mut F);
        closure(Some(value.to_string_lossy()));
    }) {
        log::error!("Error in request_input callback: {:?}", error);
    }
}

unsafe extern "C" fn request_input_cancel_trampoline<F>(user_data: *mut c_void)
where
    F: FnOnce(Option<Cow<str>>),
{
    log::debug!("request_input_cancel_trampoline");
    if let Err(error) = catch_unwind(|| {
        let closure = Box::from_raw(user_data as *mut F);
        closure(None);
    }) {
        log::error!("Error in request_input callback: {:?}", error);
    }
}
