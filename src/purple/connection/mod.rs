use super::ffi::AsPtr;
use super::{ChatConversation, Plugin};
use crate::purple;
use crate::purple::PurpleMessageFlags;
use crate::MsgInfo;
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

    pub fn serv_got_chat_in(&mut self, chat_input: MsgInfo) {
        unsafe {
            let c_sn = CString::new(chat_input.chat_sn).unwrap();
            let sn_hash = glib_sys::g_str_hash(c_sn.as_ptr() as *mut c_void);
            let who = if chat_input.author_friendly == chat_input.author_sn {
                chat_input.author_friendly
            } else {
                format!("{}!{}", chat_input.author_sn, chat_input.author_friendly)
            };
            let c_sender = CString::new(who).unwrap();
            let c_text = CString::new(chat_input.text).unwrap();

            purple_sys::serv_got_chat_in(
                self.0.as_ptr(),
                sn_hash as i32,
                c_sender.as_ptr(),
                PurpleMessageFlags::PURPLE_MESSAGE_RECV,
                c_text.as_ptr(),
                chat_input.time as i64,
            )
        }
    }

    pub fn serv_got_joined_chat(&mut self, name: &str) -> Option<ChatConversation> {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let name_hash = glib_sys::g_str_hash(c_name.as_ptr() as *mut c_void);
            let conv = purple_sys::serv_got_joined_chat(
                self.0.as_ptr(),
                name_hash as i32,
                c_name.as_ptr(),
            );
            ChatConversation::from_ptr(conv as *mut purple_sys::PurpleConvChat)
        }
    }
}

impl AsPtr for Connection {
    type PtrType = purple_sys::PurpleConnection;
    fn as_ptr(&self) -> *const purple_sys::PurpleConnection {
        self.0.as_ptr()
    }
}
