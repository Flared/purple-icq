use crate::purple::ffi::AsPtr;
use crate::purple::{Account, Connection};

pub struct Handle<T>(*mut ProtocolData<T>);

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({:?})", self.0)
    }
}

impl<T> From<&mut Connection> for Handle<T> {
    fn from(connection: &mut Connection) -> Handle<T> {
        Handle(connection.get_protocol_data() as *mut ProtocolData<T>)
    }
}

impl<T> From<&mut Account> for Handle<T> {
    fn from(account: &mut Account) -> Handle<T> {
        Handle(account.get_connection().unwrap().get_protocol_data() as *mut ProtocolData<T>)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> AsPtr for Handle<T> {
    type PtrType = ProtocolData<T>;
    fn as_ptr(&self) -> *const ProtocolData<T> {
        self.0
    }
}

impl<T> AsPtr for &Handle<T> {
    type PtrType = ProtocolData<T>;
    fn as_ptr(&self) -> *const ProtocolData<T> {
        self.0
    }
}

pub struct ProtocolData<T> {
    pub account: Account,
    pub connection: Connection,
    pub data: T,
}
