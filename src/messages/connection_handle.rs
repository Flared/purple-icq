use super::{FdSender, SystemMessage};
use crate::purple::{Connection, PurpleConnectionError, PurpleConnectionState};
use async_std::sync::channel;

pub trait AsConnection {
    unsafe fn as_connection(&self) -> Option<Connection>;
}

pub struct ConnectionProxy<'a, C: AsConnection + Clone + Send + 'static> {
    pub handle: C,
    pub sender: &'a mut FdSender<SystemMessage>,
}

impl<'a, C: AsConnection + Clone + Send + 'static> ConnectionProxy<'a, C> {
    #[allow(dead_code)]
    pub async fn exec<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(Connection) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel(1);
        self.exec_no_return(move |connection| {
            if let Err(error) = tx.try_send(f(connection)) {
                log::error!("Failed to send result: {:?}", error);
            }
        })
        .await?;
        Some(rx.recv().await.expect("Failed to receive result"))
    }

    pub async fn exec_no_return<F>(&mut self, f: F) -> Option<()>
    where
        F: FnOnce(Connection),
        F: Send + 'static,
    {
        let (tx, rx) = channel(1);
        self.sender
            .send(SystemMessage::ExecConnection {
                handle: Box::new(self.handle.clone()),
                function: Box::new(f),
                result_sender: tx,
            })
            .await;
        rx.recv().await.expect("Failed to receive result")
    }

    pub async fn set_state(&mut self, state: PurpleConnectionState) -> Option<()> {
        self.exec_no_return(move |connection| connection.set_state(state))
            .await
    }

    pub async fn error_reason(
        &mut self,
        reason: PurpleConnectionError,
        description: String,
    ) -> Option<()> {
        self.exec_no_return(move |connection| connection.error_reason(reason, &description))
            .await
    }
}
