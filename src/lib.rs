use async_std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use messages::{AccountInfo, ICQSystemHandle, PurpleMessage, SystemMessage};
use purple::*;
use std::ffi::{CStr, CString};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};

mod icq;
#[macro_use]
mod purple;
mod messages;

pub mod status {
    use lazy_static::lazy_static;
    use std::ffi::CString;
    lazy_static! {
        pub static ref ONLINE_ID: CString = CString::new("online").unwrap();
        pub static ref ONLINE_NAME: CString = CString::new("Online").unwrap();
        pub static ref OFFLINE_ID: CString = CString::new("offline").unwrap();
        pub static ref OFFLINE_NAME: CString = CString::new("Offline").unwrap();
    }
}

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
}

mod blist_node {
    pub const LAST_SEEN_TIMESTAMP: &str = "last_seen_timestamp";
}

mod chat_info {
    use lazy_static::lazy_static;
    use std::ffi::CString;
    lazy_static! {
        pub static ref SN: CString = CString::new("sn").unwrap();
        pub static ref SN_NAME: CString = CString::new("Chat ID").unwrap();
        pub static ref STAMP: CString = CString::new("stamp").unwrap();
        pub static ref TITLE: CString = CString::new("title").unwrap();
        pub static ref GROUP: CString = CString::new("group").unwrap();
        pub static ref STATE: CString = CString::new("state").unwrap();
    }
}

pub mod chat_states {
    pub const JOINED: &str = "joined";
}

#[derive(Debug, Clone)]
pub struct MsgInfo {
    pub chat_sn: String,
    pub author_sn: String,
    pub author_friendly: String,
    pub text: String,
    pub time: i64,
}

#[derive(Debug, Clone)]
pub struct ChatInfo {
    pub stamp: Option<String>,
    pub group: Option<String>,
    pub sn: String,
    pub title: String,
}

impl ChatInfo {
    pub fn from_hashtable(table: &StrHashTable) -> Option<Self> {
        Some(Self {
            stamp: table.lookup(&chat_info::STAMP).map(Into::into),
            group: table.lookup(&chat_info::GROUP).map(Into::into),
            sn: table.lookup(&chat_info::SN)?.into(),
            title: table.lookup(&chat_info::TITLE)?.into(),
        })
    }

    pub fn as_hashtable(&self) -> purple::StrHashTable {
        let mut table = purple::StrHashTable::default();
        table.insert(&chat_info::SN, &self.sn);
        if let Some(group) = &self.group {
            table.insert(&chat_info::GROUP, &group);
        }
        if let Some(stamp) = &self.stamp {
            table.insert(&chat_info::STAMP, &stamp);
        }
        table.insert(&chat_info::TITLE, &self.title);
        table
    }
}

#[derive(Debug, Default)]
pub struct AccountData {
    phone_number: String,
    session_closed: AtomicBool,
    session: RwLock<Option<icq::protocol::SessionInfo>>,
}

impl Drop for AccountData {
    fn drop(&mut self) {
        log::info!("AccountData dropped");
    }
}

pub type AccountDataBox = Arc<AccountData>;
pub type Handle = purple::Handle<AccountDataBox>;
pub type ProtocolData = purple::ProtocolData<AccountDataBox>;

pub struct PurpleICQ {
    system: ICQSystemHandle,
    connections: purple::Connections<AccountDataBox>,
    input_handle: Option<u32>,
    history_command_handle: Option<PurpleCmdId>,
}

impl purple::PrplPlugin for PurpleICQ {
    type Plugin = Self;

    fn new() -> Self {
        env_logger::init();
        let system = icq::system::spawn();
        Self {
            system,
            input_handle: None,
            history_command_handle: None,
            connections: purple::Connections::new(),
        }
    }
    fn register(&self, context: RegisterContext<Self>) -> RegisterContext<Self> {
        let info = purple::PrplInfo {
            id: "prpl-flare-icq".into(),
            name: "ICQ (Web)".into(),
            version: "0.1".into(),
            summary: "Web ICQ protocol implementation".into(),
            description: "Web ICQ protocol implementation".into(),
            author: "Israel Halle <israel.halle@flare.systems>".into(),
            homepage: "https://github.com/Flared/purple-icq".into(),
        };

        context
            .with_info(info)
            .enable_login()
            .enable_load()
            .enable_close()
            .enable_chat_info()
            .enable_chat_info_defaults()
            .enable_join_chat()
            .enable_chat_leave()
            .enable_send_im()
            .enable_chat_send()
            .enable_convo_closed()
            .enable_get_chat_name()
            .enable_list_icon()
            .enable_status_types()
    }
}

impl purple::LoginHandler for PurpleICQ {
    fn login(&mut self, account: &mut Account) {
        let phone_number = account.get_username().unwrap().into();
        let protocol_data: AccountDataBox = Arc::new(AccountData {
            phone_number,
            session_closed: AtomicBool::new(false),
            session: RwLock::new(None),
        });

        // Safe as long as we remove the account in "close".
        unsafe {
            self.connections.add(
                &mut account.get_connection().unwrap(),
                protocol_data.clone(),
            )
        };
        self.system
            .tx
            .try_send(PurpleMessage::Login(AccountInfo {
                handle: Handle::from(account),
                protocol_data,
            }))
            .unwrap();
    }
}
impl purple::CloseHandler for PurpleICQ {
    fn close(&mut self, connection: &mut Connection) {
        let handle = Handle::from(&mut *connection);
        match self.connections.get(&handle) {
            Some(protocol_data) => {
                protocol_data
                    .data
                    .session_closed
                    .store(true, Ordering::Relaxed);
                self.connections.remove(connection);
            }
            None => {
                log::error!("Tried closing a closed connection");
            }
        }
    }
}
impl purple::StatusTypeHandler for PurpleICQ {
    fn status_types(_account: &mut Account) -> Vec<StatusType> {
        vec![
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_AVAILABLE,
                Some(&status::ONLINE_ID),
                Some(&status::ONLINE_NAME),
                true,
            ),
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_OFFLINE,
                Some(&status::OFFLINE_ID),
                Some(&status::OFFLINE_NAME),
                true,
            ),
        ]
    }
}
impl purple::LoadHandler for PurpleICQ {
    fn load(&mut self, _plugin: &purple::Plugin) -> bool {
        use std::os::unix::io::AsRawFd;
        self.input_handle = Some(self.enable_input(
            self.system.input_rx.as_raw_fd(),
            purple::PurpleInputCondition::PURPLE_INPUT_READ,
        ));

        self.history_command_handle =
            Some(self.enable_command("history", "w", "history &lt;timestamp&gt;"));
        true
    }
}

impl purple::ListIconHandler for PurpleICQ {
    fn list_icon(_account: &mut Account) -> &'static CStr {
        &ICON_FILE
    }
}

impl purple::ChatInfoHandler for PurpleICQ {
    fn chat_info(&mut self, _connection: &mut Connection) -> Vec<purple::prpl::ChatEntry> {
        vec![purple::prpl::ChatEntry {
            label: &chat_info::SN_NAME,
            identifier: &chat_info::SN,
            required: true,
            is_int: false,
            min: 0,
            max: 0,
            secret: false,
        }]
    }
}

impl purple::ChatInfoDefaultsHandler for PurpleICQ {
    fn chat_info_defaults(
        &mut self,
        _connection: &mut Connection,
        chat_name: Option<&str>,
    ) -> purple::StrHashTable {
        let mut defaults = purple::StrHashTable::default();
        defaults.insert(chat_info::SN.as_c_str(), chat_name.unwrap_or(""));
        defaults
    }
}

impl purple::JoinChatHandler for PurpleICQ {
    fn join_chat(&mut self, connection: &mut Connection, data: Option<&mut StrHashTable>) {
        let data = match data {
            Some(data) => data,
            None => {
                return;
            }
        };

        let stamp = match Self::get_chat_name(Some(data)) {
            Some(stamp) => stamp,
            None => {
                log::error!("No chat name provided");
                return;
            }
        };

        if let Some(chat_states::JOINED) = data.lookup(&chat_info::STATE) {
            match ChatInfo::from_hashtable(data) {
                Some(chat_info) => {
                    self.conversation_joined(connection, &chat_info);
                    return;
                }
                None => {
                    log::error!("Unable to load chat info");
                }
            }
        }

        log::info!("Joining {}", stamp);

        let handle = Handle::from(connection);
        let protocol_data = self
            .connections
            .get(&handle)
            .expect("Tried joining chat on closed connection");

        self.system
            .tx
            .try_send(PurpleMessage::join_chat(
                handle,
                protocol_data.data.clone(),
                stamp,
            ))
            .unwrap()
    }
}

impl purple::ChatLeaveHandler for PurpleICQ {
    fn chat_leave(&mut self, _connection: &mut Connection, id: i32) {
        log::info!("Chat leave: {}", id)
    }
}

impl purple::ConvoClosedHandler for PurpleICQ {
    fn convo_closed(&mut self, _connection: &mut Connection, who: Option<&str>) {
        log::info!("Convo closed: {:?}", who)
    }
}

impl purple::GetChatNameHandler for PurpleICQ {
    fn get_chat_name(data: Option<&mut purple::StrHashTable>) -> Option<String> {
        data.and_then(|h| h.lookup(chat_info::SN.as_c_str()).map(Into::into))
    }
}

impl purple::SendIMHandler for PurpleICQ {
    fn send_im(
        &mut self,
        _connection: &mut Connection,
        _who: &str,
        _message: &str,
        _flags: PurpleMessageFlags,
    ) -> i32 {
        log::warn!("SendIM is not implemented");
        -1
    }
}

impl purple::ChatSendHandler for PurpleICQ {
    fn chat_send(
        &mut self,
        connection: &mut Connection,
        id: i32,
        message: &str,
        flags: PurpleMessageFlags,
    ) -> i32 {
        log::info!("{}: {} [{:?}]", id, message, flags);
        let mut conversation = match ChatConversation::find(connection, id) {
            Some(c) => c,
            None => {
                log::error!("Conversation not found");
                return -1;
            }
        };

        let sn = match conversation.get_data("sn") {
            Some(sn) => sn,
            None => {
                log::error!("SN not found");
                return -1;
            }
        };

        let handle = Handle::from(connection);
        let protocol_data = self.connections.get(&handle).expect("Connection closed");
        self.system
            .tx
            .try_send(PurpleMessage::send_msg(
                handle,
                protocol_data.data.clone(),
                sn.into(),
                message.into(),
            ))
            .unwrap();
        1
    }
}

impl purple::InputHandler for PurpleICQ {
    fn input(&mut self, _fd: i32, _cond: purple::PurpleInputCondition) {
        log::debug!("Input");
        // Consume the byte from the input pipe.
        let mut buf = [0; 1];
        self.system
            .input_rx
            .read_exact(&mut buf)
            .expect("Failed to read input pipe");

        // Consume the actual message.
        match self.system.rx.try_recv() {
            Ok(message) => self.process_message(message),
            Err(async_std::sync::TryRecvError::Empty) => log::error!("Expected message, but empty"),
            Err(async_std::sync::TryRecvError::Disconnected) => {
                log::error!("System disconnected");
                if let Some(input_handle) = self.input_handle {
                    self.disable_input(input_handle);
                }
            }
        };
    }
}

impl purple::CommandHandler for PurpleICQ {
    fn command(
        &mut self,
        conversation: &mut ChatConversation,
        command: &str,
        args: &[&str],
    ) -> PurpleCmdRet {
        log::error!(
            "cmd_func: conv={} cmd={} args={:?}",
            conversation.get_title().unwrap_or("unknown"),
            command,
            args
        );
        PurpleCmdRet::PURPLE_CMD_RET_OK
    }
}

impl PurpleICQ {
    fn process_message(&mut self, message: SystemMessage) {
        match message {
            SystemMessage::ExecAccount { handle, function } => {
                self.connections
                    .get(handle)
                    .map(|protocol_data| function(&mut protocol_data.account))
                    .or_else(|| {
                        log::warn!("The account connection has been closed");
                        None
                    });
            }
            SystemMessage::ExecConnection { handle, function } => {
                self.connections
                    .get(handle)
                    .map(|protocol_data| function(&mut protocol_data.connection))
                    .or_else(|| {
                        log::warn!("The account connection has been closed");
                        None
                    });
            }
            SystemMessage::ExecHandle { handle, function } => {
                self.connections
                    .get(handle)
                    .map(|mut protocol_data| function(self, &mut protocol_data))
                    .or_else(|| {
                        log::warn!("The account connection has been closed");
                        None
                    });
            }
        }
    }

    pub fn serv_got_chat_in(&mut self, connection: &mut Connection, msg_info: MsgInfo) {
        match purple::Chat::find(&mut connection.get_account(), &msg_info.chat_sn) {
            Some(mut chat) => {
                let mut node = chat.as_blist_node();
                let last_timestamp: i64 = node
                    .get_string(&blist_node::LAST_SEEN_TIMESTAMP)
                    .and_then(|t| t.parse::<i64>().ok())
                    .unwrap_or(0);
                let new_timestamp = msg_info.time;
                if new_timestamp > last_timestamp {
                    node.set_string(&blist_node::LAST_SEEN_TIMESTAMP, &new_timestamp.to_string());
                    self.conversation_joined(
                        connection,
                        &ChatInfo {
                            group: None,
                            sn: msg_info.chat_sn.clone(),
                            stamp: None,
                            title: msg_info.author_friendly.clone(),
                        },
                    );
                }
            }
            None => {
                // Don't log errors for DMs because they are not yet supported.
                // It happens all the time.
                if msg_info.chat_sn.ends_with("@chat.agent") {
                    log::error!("Got message for unknown chat {}", msg_info.chat_sn);
                }
            }
        }

        connection.serv_got_chat_in(msg_info);
    }

    pub fn chat_joined(&mut self, connection: &mut Connection, info: &ChatInfo) {
        log::info!("chat joined: {}", info.sn);
        if info.sn.ends_with("@chat.agent") {
            self.group_chat_joined(connection, info)
        } else {
            //todo!()
        }
    }

    fn group_chat_joined(&mut self, connection: &mut Connection, info: &ChatInfo) {
        let mut account = connection.get_account();
        match purple::Chat::find(&mut account, &info.sn) {
            Some(mut chat) => {
                // The chat already exists.

                // Should we replace the blist group?
                if let Some(info_group) = &info.group {
                    let should_replace_group = {
                        match chat.get_group() {
                            Some(mut chat_group) => !chat_group.get_name().eq(info_group),
                            None => true,
                        }
                    };
                    if should_replace_group {
                        chat.add_to_blist(&mut self.get_or_create_group(Some(&info_group)), None);
                    }
                }

                // Replace the alias
                chat.set_alias(&info.title);
            }
            None => {
                let mut components = info.as_hashtable();
                components.insert(&chat_info::STATE, chat_states::JOINED);
                let mut chat = purple::Chat::new(&mut account, &info.title, components);
                chat.add_to_blist(&mut self.get_or_create_group(info.group.as_deref()), None);
            }
        }
    }

    fn get_or_create_group(&mut self, name: Option<&str>) -> purple::Group {
        let name = name.unwrap_or("ICQ");
        Group::find(name).unwrap_or_else(|| {
            let mut group = purple::Group::new(name);
            group.add_to_blist(None);
            group
        })
    }

    pub fn conversation_joined(&mut self, connection: &mut Connection, info: &ChatInfo) {
        match connection.get_account().find_chat_conversation(&info.sn) {
            Some(mut conversation) => {
                if conversation.has_left() {
                    log::error!("Trying to join left conversation");
                } else {
                    conversation.present();
                }
            }
            None => {
                let mut conversation = connection.serv_got_joined_chat(&info.sn).unwrap();
                conversation.set_data("sn", &info.sn);
                if let Some(stamp) = &info.stamp {
                    conversation.set_data("stamp", stamp);
                }
                conversation.set_title(&info.title);
            }
        }
    }
}

purple_prpl_plugin!(PurpleICQ);
