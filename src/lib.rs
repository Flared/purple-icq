use async_std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use messages::{AccountInfo, ICQSystemHandle, PurpleMessage, SystemMessage};
use purple::*;
use std::collections::HashMap;
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
}

#[derive(Debug, Default)]
pub struct AccountData {
    phone_number: String,
    session: Option<icq::protocol::SessionInfo>,
}

pub type AccountDataBox = Arc<Mutex<AccountData>>;

pub type Handle = purple::Handle<AccountDataBox>;

struct PurpleICQ {
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
        _chat_name: Option<&str>,
    ) -> HashMap<&'static CStr, String> {
        let mut defaults = HashMap::new();
        defaults.insert(CHAT_INFO_SN.as_c_str(), "test-default".into());
        defaults
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

impl purple::JoinChatHandler for PurpleICQ {
    fn join_chat(&mut self, _connection: &mut Connection, data: &HashMap<String, String>) {
        let sn = data.get("sn");
        log::info!("Joining chat: {:?}", sn);
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
        }
    }
}

purple_prpl_plugin!(PurpleICQ);
