use self::account_proxy::AccountProxy;
use self::connection_proxy::ConnectionProxy;
use self::handle_proxy::HandleProxy;
use crate::purple::{Account, Connection};
use crate::{AccountDataBox, Handle, ProtocolData, PurpleICQ};
use async_std::sync::{Receiver, Sender};

mod account_proxy;
mod connection_proxy;
mod handle_proxy;

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

    pub fn try_send(&mut self, item: T) {
        self.channel_sender.try_send(item).unwrap();
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

    pub fn handle_proxy<'a>(&'a mut self, handle: &Handle) -> HandleProxy<'a> {
        HandleProxy {
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
pub struct SendMsgMessageData {
    pub to_sn: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GetChatInfoMessageData {
    pub sn: String,
}

#[derive(Debug, Clone)]
pub struct GetHistoryMessageData {
    pub sn: String,
    pub from_msg_id: String,
    pub count: i32,
}

#[derive(Debug)]
pub enum PurpleMessage {
    Login(AccountInfo),
    JoinChat(JoinChatMessage),
    SendMsg(SendMsgMessage),
    GetChatInfo(GetChatInfoMessage),
    GetHistory(GetHistoryMessage),
}

pub type JoinChatMessage = PurpleMessageWithHandle<JoinChatMessageData>;
pub type GetHistoryMessage = PurpleMessageWithHandle<GetHistoryMessageData>;
pub type SendMsgMessage = PurpleMessageWithHandle<SendMsgMessageData>;
pub type GetChatInfoMessage = PurpleMessageWithHandle<GetChatInfoMessageData>;

impl PurpleMessage {
    pub fn join_chat(handle: Handle, protocol_data: AccountDataBox, stamp: String) -> Self {
        Self::JoinChat(JoinChatMessage {
            handle,
            protocol_data,
            message_data: JoinChatMessageData { stamp },
        })
    }

    pub fn fetch_history(
        handle: Handle,
        protocol_data: AccountDataBox,
        sn: String,
        from_msg_id: String,
        count: i32,
    ) -> Self {
        Self::GetHistory(GetHistoryMessage {
            handle,
            protocol_data,
            message_data: GetHistoryMessageData {
                sn,
                from_msg_id,
                count,
            },
        })
    }

    pub fn send_msg(
        handle: Handle,
        protocol_data: AccountDataBox,
        to_sn: String,
        message: String,
    ) -> Self {
        Self::SendMsg(SendMsgMessage {
            handle,
            protocol_data,
            message_data: SendMsgMessageData { to_sn, message },
        })
    }

    pub fn get_chat_info(handle: Handle, protocol_data: AccountDataBox, sn: String) -> Self {
        Self::GetChatInfo(GetChatInfoMessage {
            handle,
            protocol_data,
            message_data: GetChatInfoMessageData { sn },
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
    ExecHandle {
        handle: Handle,
        function: Box<dyn FnOnce(&mut PurpleICQ, &mut ProtocolData) + Send + 'static>,
    },
    FlushLogs,
}

pub struct ICQSystemHandle {
    pub input_rx: os_pipe::PipeReader,
    pub rx: Receiver<SystemMessage>,
    pub tx: Sender<PurpleMessage>,
}
