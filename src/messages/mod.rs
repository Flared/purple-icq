pub use self::account_handle::AccountHandle;
use crate::purple::Account;
use async_std::sync::{Receiver, Sender};

mod account_handle;

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
        self.os_sender.write_all(&[0]).unwrap();
    }
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
        function: Box<dyn FnOnce(Account) + Send + 'static>,
    },
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: Receiver<SystemMessage>,
    pub tx: Sender<PurpleMessage>,
}
