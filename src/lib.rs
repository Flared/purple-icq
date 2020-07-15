mod glib;
mod icq;
#[macro_use]
mod purple;
mod messages;

use lazy_static::lazy_static;
use messages::{AccountHandle, AccountInfo, ICQSystemHandle, PurpleMessage, SystemMessage};
use purple::*;
use std::ffi::{CStr, CString};
use std::io::Read;

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
}

struct PurpleICQ {
    system: ICQSystemHandle,
}

impl purple::PrplPlugin for PurpleICQ {
    type Plugin = Self;
    fn new() -> Self {
        env_logger::init();
        let system = icq::system::spawn();
        Self { system }
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
            .enable_list_icon()
            .enable_status_types()
    }
}

impl purple::LoginHandler for PurpleICQ {
    fn login(&self, account: &Account) {
        let phone_number = account.get_username().unwrap().into();
        self.system
            .tx
            .try_send(PurpleMessage::Login(AccountInfo::new(
                AccountHandle::from(account),
                phone_number,
            )))
            .unwrap();
    }
}
impl purple::CloseHandler for PurpleICQ {
    fn close(&self, _connection: &Connection) {
        println!("Close");
    }
}
impl purple::StatusTypeHandler for PurpleICQ {
    fn status_types(_account: &Account) -> Vec<StatusType> {
        vec![
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_AVAILABLE,
                "online".into(),
                "Online".into(),
                true,
            ),
            StatusType::new(
                PurpleStatusPrimitive::PURPLE_STATUS_OFFLINE,
                "offline".into(),
                "Offline".into(),
                true,
            ),
        ]
    }
}
impl purple::LoadHandler for PurpleICQ {
    fn load(&mut self, _plugin: &purple::Plugin) -> bool {
        use std::os::unix::io::AsRawFd;
        self.enable_input(
            self.system.input_rx.as_raw_fd(),
            purple::PurpleInputCondition::PURPLE_INPUT_READ,
        );
        true
    }
}

impl purple::ListIconHandler for PurpleICQ {
    fn list_icon(_account: &Account) -> &'static CStr {
        &ICON_FILE
    }
}

impl purple::ChatInfoHandler for PurpleICQ {}

impl purple::InputHandler for PurpleICQ {
    fn input(&mut self, _fd: i32, _cond: purple::PurpleInputCondition) {
        log::debug!("Input");
        // Consume the byte from the input pipe.
        let mut buf = [0; 1];
        self.system
            .input_rx
            .read(&mut buf)
            .expect("Failed to read input pipe");

        // Consume the actual message.
        match self.system.rx.try_recv() {
            Ok(message) => self.process_message(message),
            Err(async_std::sync::TryRecvError::Empty) => log::error!("Expected message, but empty"),
            Err(async_std::sync::TryRecvError::Disconnected) => log::error!("System disconnected"),
        };
    }
}

impl PurpleICQ {
    fn process_message(&self, message: SystemMessage) {
        match message {
            SystemMessage::ExecAccount { handle, function } => {
                function(handle.as_account());
            }
            _ => {}
        }
    }
}

purple_prpl_plugin!(PurpleICQ);
