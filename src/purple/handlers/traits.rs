use super::super::{Account, Connection, Plugin, StatusType};
use std::ffi::CStr;
pub trait LoadHandler {
    fn load(&mut self, plugin: &Plugin) -> bool;
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

pub trait InputHandler {
    fn input(&mut self, fd: i32, cond: crate::purple::PurpleInputCondition);
    fn enable_input(&mut self, fd: i32, cond: crate::purple::PurpleInputCondition) -> u32
    where
        Self: 'static,
    {
        let self_ptr: *mut Self = self;
        crate::purple::input_add(fd, cond, move |fd, cond| {
            let this = unsafe { &mut *self_ptr };
            this.input(fd, cond);
        })
    }

    fn disable_input(&self, input_handle: u32) -> bool {
        unsafe { purple_sys::purple_input_remove(input_handle) != 0 }
    }
}
