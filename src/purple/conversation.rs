use super::ffi::{AsMutPtr, AsPtr};
use super::{Connection, PurpleConvChatBuddyFlags};
use glib::translate::{FromGlib, ToGlib};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::{null_mut, NonNull};

pub struct ChatConversation(NonNull<purple_sys::PurpleConvChat>);
pub struct Conversation(NonNull<purple_sys::PurpleConversation>);

impl Conversation {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleConversation) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn find(connection: &mut Connection, id: i32) -> Option<Self> {
        unsafe { Self::from_ptr(purple_sys::purple_find_chat(connection.as_mut_ptr(), id)) }
    }

    pub fn set_title(&mut self, title: &str) {
        unsafe {
            let c_title = CString::new(title).unwrap();
            purple_sys::purple_conversation_set_title(self.as_mut_ptr(), c_title.as_ptr());
        }
    }

    pub fn get_title(&mut self) -> Option<&str> {
        unsafe {
            let c_value = purple_sys::purple_conversation_get_title(self.as_mut_ptr());
            NonNull::new(c_value as *mut c_char).map(|p| {
                CStr::from_ptr(p.as_ptr() as *const c_char)
                    .to_str()
                    .unwrap()
            })
        }
    }

    pub fn get_connection(&mut self) -> Connection {
        unsafe {
            let c_connection = purple_sys::purple_conversation_get_gc(self.as_mut_ptr());
            Connection::from_raw(c_connection).unwrap()
        }
    }

    pub fn present(&mut self) {
        unsafe { purple_sys::purple_conversation_present(self.as_mut_ptr()) }
    }

    // Unsafe since set_data check for an existing value and frees it as `T` type, while
    // being unable to ensure the freed data is really ot `T` type.
    pub unsafe fn set_data<T>(&mut self, key: &str, data: T) {
        let c_key = CString::new(key).unwrap();
        let existing_ptr =
            purple_sys::purple_conversation_get_data(self.as_mut_ptr(), c_key.as_ptr());
        if !existing_ptr.is_null() {
            Box::<T>::from_raw(existing_ptr as *mut T);
        }

        let data_ptr = Box::into_raw(Box::new(data));
        purple_sys::purple_conversation_set_data(
            self.as_mut_ptr(),
            c_key.as_ptr(),
            data_ptr as *mut c_void,
        );
    }

    // Unsafe since get_data doesn't validate the data stored at `key` is really of `T`
    // type.
    pub unsafe fn get_data<'a, T>(&'a mut self, key: &'_ str) -> Option<&'a mut T> {
        let c_key = CString::new(key).unwrap();
        NonNull::new(purple_sys::purple_conversation_get_data(
            self.as_mut_ptr(),
            c_key.as_ptr(),
        ))
        .map(|p| &mut *(p.as_ptr() as *mut T))
    }

    // Unsafe since it doesn't validate the data stored at `key` is really of `T`
    // type.
    pub unsafe fn remove_data<T>(&mut self, key: &str) {
        let c_key = CString::new(key).unwrap();
        let existing_ptr =
            purple_sys::purple_conversation_get_data(self.as_mut_ptr(), c_key.as_ptr());
        if !existing_ptr.is_null() {
            Box::<T>::from_raw(existing_ptr as *mut T);
            purple_sys::purple_conversation_set_data(self.as_mut_ptr(), c_key.as_ptr(), null_mut());
        }
    }

    pub fn get_chat_data(&mut self) -> Option<ChatConversation> {
        unsafe {
            ChatConversation::from_ptr(purple_sys::purple_conversation_get_chat_data(
                self.as_mut_ptr(),
            ))
        }
    }
}

impl ChatConversation {
    pub unsafe fn from_ptr(ptr: *mut purple_sys::PurpleConvChat) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    pub fn has_left(&mut self) -> bool {
        FromGlib::from_glib(unsafe { purple_sys::purple_conv_chat_has_left(self.0.as_ptr()) })
    }

    pub fn add_user(
        &mut self,
        user: &str,
        extra_msg: &str,
        flags: PurpleConvChatBuddyFlags,
        new_arrival: bool,
    ) {
        let c_user = CString::new(user).unwrap();
        let c_extra_msg = CString::new(extra_msg).unwrap();
        log::info!(
            "{:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            self.as_mut_ptr(),
            c_user,
            c_user.as_ptr(),
            c_extra_msg,
            c_extra_msg.as_ptr(),
            flags,
            new_arrival.to_glib()
        );
        unsafe {
            purple_sys::purple_conv_chat_add_user(
                self.as_mut_ptr(),
                c_user.as_ptr(),
                c_extra_msg.as_ptr(),
                flags,
                new_arrival.to_glib(),
            )
        }
        log::info!("Added user");
    }

    pub fn clear_users(&mut self) {
        unsafe { purple_sys::purple_conv_chat_clear_users(self.as_mut_ptr()) }
    }

    pub fn get_conversation(&mut self) -> Conversation {
        unsafe {
            Conversation::from_ptr(purple_sys::purple_conv_chat_get_conversation(
                self.as_mut_ptr(),
            ))
            .unwrap()
        }
    }
}

impl AsPtr for Conversation {
    type PtrType = purple_sys::PurpleConversation;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
impl AsPtr for ChatConversation {
    type PtrType = purple_sys::PurpleConvChat;
    fn as_ptr(&self) -> *const Self::PtrType {
        self.0.as_ptr()
    }
}
