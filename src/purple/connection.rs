use super::Plugin;
use crate::purple;
use std::ffi::CString;
use std::ptr::NonNull;
pub struct Connection(NonNull<purple_sys::PurpleConnection>);

impl Connection {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleConnection) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn as_ptr(&mut self) -> NonNull<purple_sys::PurpleConnection> {
        self.0
    }

    pub fn get_protocol_plugin(&self) -> Option<Plugin> {
        let plugin_ptr = unsafe { purple_sys::purple_connection_get_prpl(self.0.as_ptr()) };
        if plugin_ptr.is_null() {
            None
        } else {
            Some(unsafe { Plugin::from_raw(plugin_ptr) })
        }
    }

    pub fn set_state(&self, state: purple::PurpleConnectionState) {
        log::info!("Connection state: {:?}", state);
        unsafe { purple_sys::purple_connection_set_state(self.0.as_ptr(), state) };
    }

    pub fn error_reason(&self, reason: purple::PurpleConnectionError, description: &str) {
        let c_description = CString::new(description).unwrap();
        unsafe {
            purple_sys::purple_connection_error_reason(
                self.0.as_ptr(),
                reason,
                c_description.as_ptr(),
            );
        }
    }
}
