use crate::purple::Account;
use async_std::sync::{channel, Receiver, Sender};

pub struct FdSender<T> {
    os_sender: os_pipe::PipeWriter,
    channel_sender: Sender<T>,
}

impl<T> FdSender<T> {
    pub fn new(os_sender: os_pipe::PipeWriter, channel_sender: Sender<T>) -> Self {
        Self {
            os_sender,
            channel_sender,
        }
    }

    pub async fn send(&mut self, item: T) {
        self.channel_sender.send(item).await;
        use std::io::Write;
        self.os_sender.write(&[0]).unwrap();
    }
}

#[derive(Debug)]
pub enum ExecError {
    RecvError(async_std::sync::RecvError),
    DowncastError,
}

#[derive(Debug)]
pub struct AccountInfo {
    pub account: AccountHandle,
    pub phone_number: String,
}

impl AccountInfo {
    pub fn new(account: AccountHandle, phone_number: String) -> Self {
        Self {
            account,
            phone_number,
        }
    }
}

#[derive(Debug)]
pub enum PurpleMessage {
    Login(AccountInfo),
}

pub enum SystemMessage {
    Ping,
    ExecAccount {
        handle: AccountHandle,
        function:
            Box<dyn FnOnce(Account) -> Box<dyn std::any::Any + 'static + Send> + Send + 'static>,
        result_channel: Sender<Box<dyn std::any::Any + Send>>,
    },
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: Receiver<SystemMessage>,
    pub tx: Sender<PurpleMessage>,
}

#[derive(Debug, Clone)]
pub struct AccountHandle(*mut purple_sys::PurpleAccount);

// AccountHandle are safe to clone and send to other thread.
unsafe impl Send for AccountHandle {}

impl AccountHandle {
    pub fn as_account(&self) -> Account {
        unsafe { Account::from_raw(self.0) }
    }
    pub async fn exec<F, T>(
        &self,
        f: F,
        sender: &mut FdSender<SystemMessage>,
    ) -> std::result::Result<T, ExecError>
    where
        F: FnOnce(Account) -> Box<dyn std::any::Any + Send + 'static>,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel(1);
        sender
            .send(SystemMessage::ExecAccount {
                handle: self.clone(),
                function: Box::new(f),
                result_channel: tx,
            })
            .await;
        log::debug!("Sent function to purple thread");
        let result = rx.recv().await.map_err(ExecError::RecvError)?;
        result
            .downcast::<T>()
            .or(Err(ExecError::DowncastError))
            .map(|b| *b)
    }
}

impl std::convert::From<&Account> for AccountHandle {
    fn from(account: &Account) -> Self {
        return Self(account.as_ptr());
    }
}
