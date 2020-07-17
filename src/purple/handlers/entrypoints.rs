use crate::glib::GList;
use lazy_static::lazy_static;
use log::{debug, error};
use std::panic::catch_unwind;

use super::super::{Account, Connection, Plugin};
use super::traits;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
}

pub extern "C" fn actions(
    _: *mut purple_sys::PurplePlugin,
    _: *mut c_void,
) -> *mut glib_sys::GList {
    std::ptr::null_mut()
}

pub extern "C" fn load<P: traits::LoadHandler>(plugin_ptr: *mut purple_sys::PurplePlugin) -> i32 {
    match catch_unwind(|| {
        debug!("load");
        let plugin = unsafe { Plugin::from_raw(plugin_ptr) };
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
        let account = unsafe { Account::from_raw(account_ptr) };
        let plugin = account
            .get_connection()
            .expect("No connection found for account")
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.login(&account);
    }) {
        error!("Failure in login: {:?}", error)
    };
}

pub extern "C" fn chat_info<P: traits::ChatInfoHandler>(
    _: *mut purple_sys::PurpleConnection,
) -> *mut glib_sys::GList {
    debug!("chat_info");
    std::ptr::null_mut()
}

pub extern "C" fn close<P: traits::CloseHandler>(
    connection_ptr: *mut purple_sys::PurpleConnection,
) {
    if let Err(error) = catch_unwind(|| {
        debug!("close");
        let connection = unsafe { Connection::from_raw(connection_ptr).unwrap() };
        let plugin = connection
            .get_protocol_plugin()
            .expect("No plugin found for connection");
        let prpl_plugin = unsafe { plugin.extra::<P>() };
        prpl_plugin.close(&connection)
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
        let account = unsafe { Account::from_raw(account_ptr) };
        P::list_icon(&account).as_ptr()
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
        let account = unsafe { Account::from_raw(account_ptr) };
        GList::from(
            P::status_types(&account)
                .into_iter()
                .map(|s| s.into_raw() as *mut c_void)
                .collect::<Vec<*mut c_void>>(),
        )
        .into_raw()
    }) {
        Ok(r) => r,
        Err(error) => {
            error!("Failure in status_types: {:?}", error);
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn join_chat_handler(
    _: *mut purple_sys::PurpleConnection,
    _components: *mut purple_sys::GHashTable,
) {
    println!("join_chat_handler");
}
pub extern "C" fn chat_info_defaults_handler(
    _: *mut purple_sys::PurpleConnection,
    _: *const c_char,
) -> *mut purple_sys::GHashTable {
    println!("chat_info_defaults_handler");
    std::ptr::null_mut()
}
pub extern "C" fn roomlist_get_list_handler(
    _: *mut purple_sys::PurpleConnection,
) -> *mut purple_sys::PurpleRoomlist {
    println!("roomlist_get_list_handler");
    std::ptr::null_mut()
}
