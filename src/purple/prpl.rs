use glib::translate::{GlibPtrDefault, Stash, ToGlib, ToGlibPtr};
use std::ffi::CStr;

pub struct ChatEntry {
    pub label: &'static CStr,
    pub identifier: &'static CStr,
    pub required: bool,
    pub is_int: bool,
    pub min: i32,
    pub max: i32,
    pub secret: bool,
}

pub struct ProtoChatEntry(purple_sys::proto_chat_entry);

impl Into<ProtoChatEntry> for ChatEntry {
    fn into(self) -> ProtoChatEntry {
        ProtoChatEntry(purple_sys::proto_chat_entry {
            label: self.label.as_ptr(),
            identifier: self.identifier.as_ptr(),
            required: self.required.to_glib(),
            is_int: self.is_int.to_glib(),
            min: self.min,
            max: self.max,
            secret: self.secret.to_glib(),
        })
    }
}

impl GlibPtrDefault for ProtoChatEntry {
    type GlibType = *const purple_sys::proto_chat_entry;
}

impl<'a> ToGlibPtr<'a, *const purple_sys::proto_chat_entry> for ProtoChatEntry {
    type Storage = &'a ProtoChatEntry;

    fn to_glib_none(&'a self) -> Stash<'a, *const purple_sys::proto_chat_entry, Self> {
        Stash(&self.0 as *const purple_sys::proto_chat_entry, self)
    }

    fn to_glib_full(&self) -> *const purple_sys::proto_chat_entry {
        unsafe {
            let res = glib_sys::g_malloc(std::mem::size_of::<purple_sys::proto_chat_entry>())
                as *mut purple_sys::proto_chat_entry;

            *res = self.0;
            res
        }
    }
}
