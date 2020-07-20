use super::{FdSender, SystemMessage};
use crate::purple::{Connection, PurpleConnectionError, PurpleConnectionState};
use crate::Handle;
use async_std::sync::channel;

pub struct ConnectionProxy<'a> {
    pub handle: Handle,
    pub sender: &'a mut FdSender<SystemMessage>,
}

impl<'a> ConnectionProxy<'a> {
    #[allow(dead_code)]
    pub async fn exec<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Connection) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel(1);
        self.exec_no_return(move |connection| {
            if let Err(error) = tx.try_send(f(connection)) {
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
        F: FnOnce(&mut Connection),
        F: Send + 'static,
    {
        self.sender
            .send(SystemMessage::ExecConnection {
                handle: self.handle.clone(),
                function: Box::new(f),
            })
            .await;
    }

    pub async fn set_state(&mut self, state: PurpleConnectionState) {
        self.exec_no_return(move |connection| connection.set_state(state))
            .await
    }

    pub async fn error_reason(&mut self, reason: PurpleConnectionError, description: String) {
        self.exec_no_return(move |connection| connection.error_reason(reason, &description))
            .await
    }
}
