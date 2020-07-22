use super::super::{prpl, Account, Connection, Plugin};
use super::traits;
use crate::purple::ffi::ToGlibContainerFromIterator;
use glib::translate::{FromGlibPtrContainer, ToGlibContainerFromSlice};
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
        debug!("close");
        let mut connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let mut plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        let data = unsafe { FromGlibPtrContainer::from_glib_none(components) };
        prpl_plugin.join_chat(&mut connection, &data)
    }) {
        error!("Failure in close: {:?}", error)
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
        ToGlibContainerFromIterator::into_glib_full_from_iter(
            prpl_plugin.chat_info_defaults(&mut connection, chat_name.as_deref()),
        )
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
