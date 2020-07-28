pub use self::account::Account;
pub use self::blist::BlistNode;
pub use self::chat::Chat;
pub use self::connection::protocol_data::ProtocolData;
pub use self::connection::{Connection, Connections, Handle};
pub use self::conversation::Conversation;
pub use self::group::Group;
pub use self::handlers::traits::*;
pub use self::hashtable::StrHashTable;
pub use self::loader::{PrplInfo, PrplPluginLoader, RegisterContext};
pub use self::plugin::Plugin;
pub use self::status_type::{PurpleStatusPrimitive, StatusType};
use glib::translate::FromGlibPtrContainer;
pub use purple_sys;
pub use purple_sys::{
    PurpleCmdId, PurpleCmdRet, PurpleConnectionError, PurpleConnectionState,
    PurpleConvChatBuddyFlags, PurpleConversationType, PurpleInputCondition, PurpleMessageFlags,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::panic::catch_unwind;
use std::ptr;

pub mod account;
mod blist;
mod chat;
mod connection;
mod conversation;
pub mod ffi;
mod group;
mod handlers;
mod hashtable;
mod loader;
mod plugin;
pub mod prpl;
mod status_type;

pub trait PrplPlugin {
    type Plugin;
    fn new() -> Self;
    fn register(&self, context: RegisterContext<Self::Plugin>) -> RegisterContext<Self::Plugin>;
}

macro_rules! purple_prpl_plugin {
    ($plugin:ty) => {
        /// # Safety
        /// This function is the plugin entrypoints and should not be called manually.
        #[no_mangle]
        pub unsafe extern "C" fn purple_init_plugin(
            plugin_ptr: *mut purple_sys::PurplePlugin,
        ) -> i32 {
            // Safe as long as called from libpurple. Should be the
            // case since this function is called by libpurple.
            let plugin = purple::PrplPluginLoader::<$plugin>::from_raw(plugin_ptr);
            plugin.init()
        }
    };
}

pub fn input_add<F>(fd: i32, cond: PurpleInputCondition, callback: F) -> u32
where
    F: Fn(i32, PurpleInputCondition) + 'static,
{
    let user_data = Box::into_raw(Box::new(callback)) as *mut c_void;
    unsafe { purple_sys::purple_input_add(fd, cond, Some(trampoline::<F>), user_data) }
}

unsafe extern "C" fn trampoline<F>(user_data: *mut c_void, df: i32, cond: PurpleInputCondition)
where
    F: Fn(i32, PurpleInputCondition),
{
    if let Err(error) = catch_unwind(|| {
        let closure = &*(user_data as *mut F);
        closure(df, cond);
    }) {
        log::error!("Failure in input handler: {:?}", error);
    }
}

pub fn register_cmd<F>(
    cmd: &str,
    args: &str,
    help_text: &str,
    callback: F,
) -> purple_sys::PurpleCmdId
where
    F: Fn(&mut Conversation, &str, &[&str]) -> purple_sys::PurpleCmdRet + 'static,
{
    let user_data = Box::into_raw(Box::new(callback)) as *mut c_void;
    let c_cmd = CString::new(cmd).unwrap();
    let c_args = CString::new(args).unwrap();
    let c_help = CString::new(help_text).unwrap();

    unsafe {
        purple_sys::purple_cmd_register(
            c_cmd.as_ptr(),
            c_args.as_ptr(),
            purple_sys::PurpleCmdPriority::PURPLE_CMD_P_DEFAULT,
            purple_sys::PurpleCmdFlag::PURPLE_CMD_FLAG_CHAT,
            ptr::null(),
            Some(trampoline_cmd::<F>),
            c_help.as_ptr(),
            user_data,
        )
    }
}

unsafe extern "C" fn trampoline_cmd<F>(
    conversation_ptr: *mut purple_sys::PurpleConversation,
    c_cmd: *const c_char,
    c_args: *mut *mut c_char,
    _c_error: *mut *mut c_char,
    user_data: *mut c_void,
) -> purple_sys::PurpleCmdRet
where
    F: Fn(&mut Conversation, &str, &[&str]) -> purple_sys::PurpleCmdRet,
{
    let closure = &*(user_data as *mut F);

    let cmd = CStr::from_ptr(c_cmd).to_str().unwrap();
    let args: Vec<String> = FromGlibPtrContainer::from_glib_none(c_args);
    let mut conversation = Conversation::from_ptr(conversation_ptr).unwrap();

    closure(
        &mut conversation,
        cmd,
        args.iter()
            .map(std::ops::Deref::deref)
            .collect::<Vec<&str>>()
            .as_slice(),
    )
}
