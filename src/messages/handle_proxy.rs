use super::{FdSender, SystemMessage};
use crate::{Handle, ProtocolData};
use async_std::channel;

pub struct HandleProxy<'a> {
    pub handle: Handle,
    pub sender: &'a mut FdSender<SystemMessage>,
}

impl<'a> HandleProxy<'a> {
    #[allow(dead_code)]
    pub async fn exec<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut crate::PurpleICQ, &mut ProtocolData) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel::bounded(1);
        self.exec_no_return(move |plugin, protocol_data| {
            if let Err(error) = tx.try_send(f(plugin, protocol_data)) {
                log::error!("Failed to send result: {:?}", error);
            }
        })
        .await;
        rx.recv().await.ok().or_else(|| {
            log::error!("Failed to receive result");
            None
        })
    }

    pub async fn exec_no_return<F>(&mut self, f: F)
    where
        F: FnOnce(&mut crate::PurpleICQ, &mut ProtocolData),
        F: Send + 'static,
    {
        self.sender
            .send(SystemMessage::ExecHandle {
                handle: self.handle.clone(),
                function: Box::new(f),
            })
            .await;
    }
}
