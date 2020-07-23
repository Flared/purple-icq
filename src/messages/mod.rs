use self::account_proxy::AccountProxy;
use self::connection_proxy::ConnectionProxy;
use crate::purple::{Account, Connection};
use crate::{AccountDataBox, Handle};
use async_std::sync::{Receiver, Sender};

mod account_proxy;
mod connection_proxy;

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

impl FdSender<SystemMessage> {
    pub fn connection_proxy<'a>(&'a mut self, handle: &Handle) -> ConnectionProxy<'a> {
        ConnectionProxy {
            handle: handle.clone(),
            sender: self,
        }
    }

    pub fn account_proxy<'a>(&'a mut self, handle: &Handle) -> AccountProxy<'a> {
        AccountProxy {
            handle: handle.clone(),
            sender: self,
        }
    }
}

impl<T> Clone for FdSender<T> {
    fn clone(&self) -> Self {
        Self {
            os_sender: self.os_sender.try_clone().unwrap(),
            channel_sender: self.channel_sender.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub handle: Handle,
    pub protocol_data: AccountDataBox,
}

#[derive(Debug, Clone)]
pub struct PurpleMessageWithHandle<T> {
    pub handle: Handle,
    pub protocol_data: AccountDataBox,
    pub message_data: T,
}

#[derive(Debug, Clone)]
pub struct JoinChatMessageData {
    pub stamp: String,
}

#[derive(Debug, Clone)]
pub struct ChatJoinedInfo {
    pub handle: Handle,
    pub stamp: String,
    pub sn: String,
    pub title: String,
}

pub type JoinChatMessage = PurpleMessageWithHandle<JoinChatMessageData>;

#[derive(Debug)]
pub enum PurpleMessage {
    Login(AccountInfo),
    JoinChat(JoinChatMessage),
}

impl PurpleMessage {
    pub fn join_chat(handle: Handle, protocol_data: AccountDataBox, stamp: String) -> Self {
        Self::JoinChat(JoinChatMessage {
            handle,
            protocol_data,
            message_data: JoinChatMessageData { stamp },
        })
    }
}

pub enum SystemMessage {
    ExecAccount {
        handle: Handle,
        function: Box<dyn FnOnce(&mut Account) + Send + 'static>,
    },
    ExecConnection {
        handle: Handle,
        function: Box<dyn FnOnce(&mut Connection) + Send + 'static>,
    },
    ChatJoined(ChatJoinedInfo),
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: Receiver<SystemMessage>,
    pub tx: Sender<PurpleMessage>,
}
