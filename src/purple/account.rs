use super::Connection;
use purple_sys;
pub struct Account(*mut purple_sys::PurpleAccount);

impl Account {
    pub unsafe fn from_raw(ptr: *mut purple_sys::PurpleAccount) -> Self {
        Account(ptr)
    }

    pub fn get_connection(&self) -> Option<Connection> {
        let connection_ptr = unsafe { purple_sys::purple_account_get_connection(self.0) };
        if connection_ptr.is_null() {
            None
        } else {
            Some(unsafe { Connection::from_raw(connection_ptr) })
        }
    }
}
