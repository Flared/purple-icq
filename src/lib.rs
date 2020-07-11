#[macro_use]
mod purple;
mod glib;

use purple::*;
use std::ffi::{CStr, CString};
use lazy_static::lazy_static;

lazy_static! {
    static ref ICON_FILE: CString = CString::new("icq").unwrap();
}

struct PurpleICQ(String);

impl purple::PrplPlugin for PurpleICQ {
    type Plugin = Self;
    fn new() -> Self {
        env_logger::init();

        Self("Hello".into())
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
    fn login(&self, _account: &Account) {
        println!("Login");
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
    fn load(&self, _plugin: &purple::Plugin) -> bool {
        println!("load {}", self.0);
        true
    }
}

impl purple::ListIconHandler for PurpleICQ {
    fn list_icon(_account: &Account) -> &'static CStr {
        &ICON_FILE
    }
}

impl purple::ChatInfoHandler for PurpleICQ {}

purple_prpl_plugin!(PurpleICQ);
