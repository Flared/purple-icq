use crate::purple::ffi::AsMutPtr;
use crate::purple::{Account, Connection};

pub struct Handle<T>(*mut ProtocolData<T>);

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({:?})", self.0)
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

impl<T> AsMutPtr<ProtocolData<T>> for Handle<T> {
    fn as_mut_ptr(&mut self) -> *mut ProtocolData<T> {
        self.0
    }
}

pub struct ProtocolData<T> {
    pub account: Account,
    pub connection: Connection,
    pub data: T,
}
