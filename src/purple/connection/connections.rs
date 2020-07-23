use super::protocol_data::ProtocolData;
use crate::purple::ffi::AsMutPtr;
use crate::purple::Connection;
use std::os::raw::c_void;

pub struct Connections<T> {
    protocol_datas: std::collections::HashSet<*mut ProtocolData<T>>,
}

impl<T> Connections<T> {
    pub fn new() -> Self {
        Self {
            protocol_datas: Default::default(),
        }
    }

    pub unsafe fn add(&mut self, connection: &mut Connection, data: T) {
        let account = connection.get_account();
        let data_ptr = Box::new(ProtocolData::<T> {
            account,
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

    pub fn get<'b, P>(&mut self, mut ptr: P) -> Option<&'b mut ProtocolData<T>>
    where
        P: AsMutPtr<PtrType = ProtocolData<T>>,
    {
        self.protocol_datas
            .get(&ptr.as_mut_ptr())
            .cloned()
            .map(|p| unsafe { &mut *p })
    }
}
