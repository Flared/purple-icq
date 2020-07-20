use crate::purple::ffi::AsMutPtr;
use crate::purple::{Account, Connection};

#[derive(Clone, Debug)]
pub struct Handle<T>(*mut ProtocolData<T>);

impl<T> From<&mut Account> for Handle<T> {
    fn from(account: &mut Account) -> Handle<T> {
        Handle(account.get_connection().unwrap().get_protocol_data() as *mut ProtocolData<T>)
    }
}

pub struct ProtocolData<T> {
    pub account: Handle<T>,
    pub connection: Connection,
    pub data: T,
}

pub struct ProtocolDatas<T> {
    protocol_datas: std::collections::HashSet<*mut ProtocolData<T>>,
}

impl<T> Default for ProtocolDatas<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ProtocolDatas<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub unsafe fn add(&mut self, connection: &Connection, data: T) {
        let account = connection.get_account();
        let data_ptr = Box::new(ProtocolData::<T> {
            account: account,
            connection: connection.clone(),
            data,
        });
        let data_raw_ptr = Box::into_raw(data_ptr);
        connection.set_protocol_data(data_raw_ptr);
        self.protocol_datas.insert(data_raw_ptr);
    }

    pub fn remove(&mut self, connection: &Connection) {
        let protocol_data_ptr = connection.get_protocol_data();
        self.accounts.remove(&protocol_data_ptr);
        Box::from_raw(protocol_data_ptr);
    }

    pub fn get<P>(&mut self, ptr: P) -> Option<&ProtocolData<T>>
    where
        P: AsMutPtr<ProtocolData<T>>,
    {
        self.protocol_datas.get(ptr).cloned().map(|p| &*p.clone())
    }
}
