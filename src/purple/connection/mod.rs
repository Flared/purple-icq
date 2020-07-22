use super::{ChatConversation, Plugin};
use crate::purple;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;

pub mod connections;
pub mod protocol_data;

pub use self::connections::Connections;
pub use self::protocol_data::Handle;

#[derive(Clone)]
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

    pub fn set_protocol_data(&mut self, data: *mut c_void) {
        unsafe { purple_sys::purple_connection_set_protocol_data(self.0.as_ptr(), data) };
    }

    pub fn get_protocol_data(&mut self) -> *mut c_void {
        unsafe { purple_sys::purple_connection_get_protocol_data(self.0.as_ptr()) }
    }

    pub fn get_account(&mut self) -> purple::Account {
        unsafe {
            purple::Account::from_raw(purple_sys::purple_connection_get_account(self.0.as_ptr()))
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

    pub fn serv_got_joined_chat(&mut self, stamp: &str) -> Option<ChatConversation> {
        unsafe {
            let c_stamp = CString::new(stamp).unwrap();
            let stamp_hash = glib_sys::g_str_hash(c_stamp.as_ptr() as *mut c_void);
            let conv = purple_sys::serv_got_joined_chat(
                self.0.as_ptr(),
                stamp_hash as i32,
                c_stamp.as_ptr(),
            );
            ChatConversation::from_ptr(conv as *mut purple_sys::PurpleConvChat)
        }
    }
}
