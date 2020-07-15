use super::super::{Account, Connection, Plugin, StatusType};
use std::ffi::CStr;
pub trait LoadHandler {
    fn load(&self, plugin: &Plugin) -> bool;
}

pub trait LoginHandler {
    fn login(&self, account: &Account);
}

pub trait CloseHandler {
    fn close(&self, connection: &Connection);
}

pub trait StatusTypeHandler {
    fn status_types(account: &Account) -> Vec<StatusType>;
}

pub trait ListIconHandler {
    fn list_icon(account: &Account) -> &'static CStr;
}
pub trait ChatInfoHandler {}
