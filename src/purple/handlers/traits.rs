use super::super::{prpl, Account, Connection, Plugin, StatusType};
use std::collections::HashMap;
use std::ffi::CStr;
pub trait LoadHandler {
    fn load(&mut self, plugin: &Plugin) -> bool;
}

pub trait LoginHandler {
    fn login(&mut self, account: &mut Account);
}

pub trait CloseHandler {
    fn close(&mut self, connection: &mut Connection);
}

pub trait StatusTypeHandler {
    fn status_types(account: &mut Account) -> Vec<StatusType>;
}

pub trait ListIconHandler {
    fn list_icon(account: &mut Account) -> &'static CStr;
}

pub trait ChatInfoHandler {
    fn chat_info(&mut self, connection: &mut Connection) -> Vec<prpl::ChatEntry>;
}

pub trait ChatInfoDefaultsHandler {
    fn chat_info_defaults(
        &mut self,
        connection: &mut Connection,
        chat_name: Option<&str>,
    ) -> HashMap<&'static CStr, String>;
}

pub trait JoinChatHandler {
    fn join_chat(&mut self, connection: &mut Connection, data: &HashMap<String, String>);
}

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
