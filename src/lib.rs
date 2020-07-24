use async_std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use messages::{AccountInfo, ICQSystemHandle, PurpleMessage, SystemMessage};
use purple::*;
use std::ffi::{CStr, CString};
use std::io::Read;

mod icq;
#[macro_use]
mod purple;
mod messages;

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
    static ref STATUS_ONLINE_ID: CString = CString::new("online").unwrap();
    static ref STATUS_ONLINE_NAME: CString = CString::new("Online").unwrap();
    static ref STATUS_OFFLINE_ID: CString = CString::new("offline").unwrap();
    static ref STATUS_OFFLINE_NAME: CString = CString::new("Offline").unwrap();
    static ref CHAT_INFO_SN: CString = CString::new("sn").unwrap();
    static ref CHAT_INFO_SN_NAME: CString = CString::new("Chat ID").unwrap();
    static ref CHAT_INFO_STAMP: CString = CString::new("stamp").unwrap();
    static ref CHAT_INFO_TITLE: CString = CString::new("title").unwrap();
    static ref CHAT_INFO_GROUP: CString = CString::new("group").unwrap();
    static ref CHAT_INFO_STATE: CString = CString::new("state").unwrap();
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
            stamp: table.lookup(&CHAT_INFO_STAMP).map(Into::into),
            group: table.lookup(&CHAT_INFO_GROUP).map(Into::into),
            sn: table.lookup(&CHAT_INFO_SN)?.into(),
            title: table.lookup(&CHAT_INFO_TITLE)?.into(),
        })
    }

    pub fn as_hashtable(&self) -> purple::StrHashTable {
        let mut table = purple::StrHashTable::default();
        table.insert(&CHAT_INFO_SN, &self.sn);
        if let Some(group) = &self.group {
            table.insert(&CHAT_INFO_GROUP, &group);
        }
        if let Some(stamp) = &self.stamp {
            table.insert(&CHAT_INFO_STAMP, &stamp);
        }
        table.insert(&CHAT_INFO_TITLE, &self.title);
        table
    }
}

#[derive(Debug, Default)]
pub struct AccountData {
    phone_number: String,
    session: Option<icq::protocol::SessionInfo>,
}

pub type AccountDataBox = Arc<Mutex<AccountData>>;
pub type Handle = purple::Handle<AccountDataBox>;
pub type ProtocolData = purple::ProtocolData<AccountDataBox>;

pub struct PurpleICQ {
    system: ICQSystemHandle,
    connections: purple::Connections<AccountDataBox>,
    input_handle: Option<u32>,
}

impl purple::PrplPlugin for PurpleICQ {
    type Plugin = Self;

    fn new() -> Self {
        env_logger::init();
        let system = icq::system::spawn();
        Self {
            system,
            input_handle: None,
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
        let protocol_data: AccountDataBox = Arc::new(Mutex::new(AccountData {
            phone_number,
            session: None,
        }));

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
        self.connections.remove(connection);
    }
}
impl purple::StatusTypeHandler for PurpleICQ {
    fn status_types(_account: &mut Account) -> Vec<StatusType> {
        vec![
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_AVAILABLE,
                Some(&STATUS_ONLINE_ID),
                Some(&STATUS_ONLINE_NAME),
                true,
            ),
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_OFFLINE,
                Some(&STATUS_OFFLINE_ID),
                Some(&STATUS_OFFLINE_NAME),
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
            label: &CHAT_INFO_SN_NAME,
            identifier: &CHAT_INFO_SN,
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
        defaults.insert(CHAT_INFO_SN.as_c_str(), chat_name.unwrap_or(""));
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

        if let Some("joined") = data.lookup(&CHAT_INFO_STATE) {
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
        data.and_then(|h| h.lookup(CHAT_INFO_SN.as_c_str()).map(Into::into))
    }
}

impl purple::SendIMHandler for PurpleICQ {
    fn send_im(
        &mut self,
        _connection: &mut Connection,
        who: &str,
        message: &str,
        flags: purple::PurpleMessageFlags,
    ) -> i32 {
        log::info!("{}: {} [{:?}]", who, message, flags);
        1
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

    pub fn chat_joined(&mut self, connection: &mut Connection, info: &ChatInfo) {
        log::info!("chat joined: {}", info.sn);
        if info.sn.ends_with("@chat.agent") {
            self.group_chat_joined(connection, info)
        } else {
            todo!()
        }
    }

    fn group_chat_joined(&mut self, connection: &mut Connection, info: &ChatInfo) {
        // Chat already joined.
        let mut account = connection.get_account();
        let chat = purple::Chat::find(&mut account, &info.sn);
        if chat.is_some() {
            // TODO: Merge existing account info
            return;
        }

        let mut components = info.as_hashtable();
        components.insert(&CHAT_INFO_STATE, "joined");
        let mut chat = purple::Chat::new(&mut account, &info.title, components);
        chat.add_to_blist(&mut self.icq_group(info.group.as_deref()), None);
    }

    fn icq_group(&mut self, name: Option<&str>) -> purple::Group {
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
