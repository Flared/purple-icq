use crate::purple::ffi::AsMutPtr;
use crate::purple::{Account, Connection};
use std::os::raw::c_void;

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

pub struct ProtocolDatas<T> {
    protocol_datas: std::collections::HashSet<*mut ProtocolData<T>>,
}

impl<T> ProtocolDatas<T> {
    pub fn new() -> Self {
        Self {
            protocol_datas: Default::default(),
        }
    }

    pub unsafe fn add(&mut self, connection: &mut Connection, data: T) {
        let account = connection.get_account();
        let data_ptr = Box::new(ProtocolData::<T> {
            account: account,
            connection: connection.clone(),
            data,
        });
        let data_raw_ptr = Box::into_raw(data_ptr);
        connection.set_protocol_data(data_raw_ptr as *mut c_void);
        self.protocol_datas.insert(data_raw_ptr);
    }

    pub fn remove(&mut self, connection: &mut Connection) {
        let protocol_data_ptr = connection.get_protocol_data() as *mut ProtocolData<T>;
        self.protocol_datas.remove(&protocol_data_ptr);
        // Retake ownership of the protocol data to drop its data.
        unsafe { Box::from_raw(protocol_data_ptr) };
    }

    pub fn get<P>(&mut self, mut ptr: P) -> Option<&mut ProtocolData<T>>
    where
        P: AsMutPtr<ProtocolData<T>>,
    {
        self.protocol_datas
            .get(&ptr.as_mut_ptr())
            .cloned()
            .map(|p| unsafe { &mut *p.clone() })
    }
}
