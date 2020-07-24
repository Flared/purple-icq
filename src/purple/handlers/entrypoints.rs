use super::super::{prpl, Account, Connection, Plugin, PurpleMessageFlags, StrHashTable};
use super::traits;
use crate::purple::ffi::{IntoGlibPtr, ToGlibContainerFromIterator};
use glib::translate::{ToGlibContainerFromSlice, ToGlibPtr};
use log::{debug, error};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::panic::catch_unwind;
use std::ptr::NonNull;

pub extern "C" fn actions(
    _: *mut purple_sys::PurplePlugin,
    _: *mut c_void,
) -> *mut glib_sys::GList {
    std::ptr::null_mut()
}

pub extern "C" fn load<P: traits::LoadHandler>(plugin_ptr: *mut purple_sys::PurplePlugin) -> i32 {
    match catch_unwind(|| {
        debug!("load");
        let mut plugin = unsafe { Plugin::from_raw(plugin_ptr) };
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.load(&plugin) as i32
    }) {
        Ok(r) => r,
        Err(_) => 0,
    }
}

pub extern "C" fn login<P: traits::LoginHandler>(account_ptr: *mut purple_sys::PurpleAccount) {
    if let Err(error) = catch_unwind(|| {
        debug!("login");
        let mut account = unsafe { Account::from_raw(account_ptr) };
        let mut plugin = account
            .get_connection()
            .expect("No connection found for account")
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.login(&mut account);
    }) {
        error!("Failure in login: {:?}", error)
    };
}

pub extern "C" fn chat_info<P: traits::ChatInfoHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
) -> *mut glib_sys::GList {
    match catch_unwind(|| {
        debug!("chat_info");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        ToGlibContainerFromSlice::to_glib_full_from_slice(
            &prpl_plugin
                .chat_info(&mut connection)
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<prpl::ProtoChatEntry>>(),
        )
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in chat_info: {:?}", error);
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn close<P: traits::CloseHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
) {
    if let Err(error) = catch_unwind(|| {
        debug!("close");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.close(&mut connection)
    }) {
        error!("Failure in close: {:?}", error)
    }
}

pub extern "C" fn list_icon<P: traits::ListIconHandler>(
    account_ptr: *mut purple_sys::PurpleAccount,
    _: *mut purple_sys::PurpleBuddy,
) -> *const c_char {
    match catch_unwind(|| {
        debug!("list_icon");
        let mut account = unsafe { Account::from_raw(account_ptr) };
        P::list_icon(&mut account).as_ptr()
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in list_icon: {:?}", error);
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn status_types<P: traits::StatusTypeHandler>(
    account_ptr: *mut purple_sys::PurpleAccount,
) -> *mut glib_sys::GList {
    match catch_unwind(|| {
        debug!("status_types");
        let mut account = unsafe { Account::from_raw(account_ptr) };
        ToGlibContainerFromIterator::into_glib_full_from_iter(P::status_types(&mut account))
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in status_types: {:?}", error);
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn join_chat<P: traits::JoinChatHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    components: *mut glib_sys::GHashTable,
) {
    if let Err(error) = catch_unwind(|| {
        debug!("join_chat");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        let mut data = unsafe { StrHashTable::from_ptr(components) };
        prpl_plugin.join_chat(&mut connection, data.as_mut())
    }) {
        error!("Failure in join_chat: {:?}", error)
    }
}

pub extern "C" fn chat_leave<P: traits::ChatLeaveHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    id: i32,
) {
    if let Err(error) = catch_unwind(|| {
        debug!("chat_leave");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.chat_leave(&mut connection, id)
    }) {
        error!("Failure in chat_leave: {:?}", error)
    }
}

pub extern "C" fn convo_closed<P: traits::ConvoClosedHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    c_who: *const c_char,
) {
    if let Err(error) = catch_unwind(|| {
        debug!("convo_closed");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        let who = NonNull::new(c_who as *mut _)
            .map(|p| unsafe { CStr::from_ptr(p.as_ptr()).to_str().unwrap() });
        prpl_plugin.convo_closed(&mut connection, who)
    }) {
        error!("Failure in convo_closed: {:?}", error)
    }
}

pub extern "C" fn get_chat_name<P: traits::GetChatNameHandler>(
    data: *mut glib_sys::GHashTable,
) -> *mut c_char {
    match catch_unwind(|| {
        debug!("close");
        let mut data = unsafe { StrHashTable::from_ptr(data) };
        let name = P::get_chat_name(data.as_mut());
        name.to_glib_full()
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in get_chat_name: {:?}", error);
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn chat_info_defaults<P: traits::ChatInfoDefaultsHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    c_chat_name: *const c_char,
) -> *mut glib_sys::GHashTable {
    match catch_unwind(|| {
        debug!("chat_info_defaults_handler");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };

        let chat_name = NonNull::new(c_chat_name as *mut _)
            .map(|p| unsafe { CStr::from_ptr(p.as_ptr()).to_string_lossy() });
        prpl_plugin
            .chat_info_defaults(&mut connection, chat_name.as_deref())
            .into_glib_full()
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in chat_info_defaults: {:?}", error);
            std::ptr::null_mut()
        }
    }
}
pub extern "C" fn roomlist_get_list_handler(
    _: *mut purple_sys::PurpleConnection,
) -> *mut purple_sys::PurpleRoomlist {
    println!("roomlist_get_list_handler");
    std::ptr::null_mut()
}

pub extern "C" fn chat_send<P: traits::ChatSendHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    id: i32,
    c_message: *const c_char,
    flags: PurpleMessageFlags,
) -> i32 {
    match catch_unwind(|| {
        debug!("convo_closed");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        let message = unsafe { CStr::from_ptr(c_message).to_str().unwrap() };
        prpl_plugin.chat_send(&mut connection, id, message, flags)
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in convo_closed: {:?}", error);
            -1
        }
    }
}
pub extern "C" fn send_im<P: traits::SendIMHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
    c_who: *const c_char,
    c_message: *const c_char,
    flags: PurpleMessageFlags,
) -> i32 {
    match catch_unwind(|| {
        debug!("convo_closed");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        let who = unsafe { CStr::from_ptr(c_who).to_str().unwrap() };
        let message = unsafe { CStr::from_ptr(c_message).to_str().unwrap() };
        prpl_plugin.send_im(&mut connection, who, message, flags)
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in convo_closed: {:?}", error);
            -1
        }
    }
}
