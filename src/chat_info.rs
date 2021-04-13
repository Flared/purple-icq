use super::icq;
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

#[derive(Debug, Clone)]
pub struct MemberRole(String);

#[derive(Debug, Clone, Default)]
pub struct PartialChatInfo {
    pub sn: String,
    pub title: String,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ChatInfo {
    pub stamp: Option<String>,
    pub group: Option<String>,
    pub sn: String,
    pub title: String,
    pub about: Option<String>,
    pub members_version: String,
    pub info_version: String,
    pub members: Vec<ChatMember>,
}

#[derive(Debug, Clone)]
pub struct ChatMember {
    pub sn: String,
    pub friendly_name: Option<String>,
    pub role: MemberRole,
    pub last_seen: Option<u64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatInfoVersion {
    pub members_version: String,
    pub info_version: String,
}

impl MemberRole {
    pub fn as_flags(&self) -> purple::PurpleConvChatBuddyFlags {
        match self.0.as_str() {
            "admin" => purple::PurpleConvChatBuddyFlags::PURPLE_CBFLAGS_OP,
            "readonly" => purple::PurpleConvChatBuddyFlags::PURPLE_CBFLAGS_NONE,
            _ => purple::PurpleConvChatBuddyFlags::PURPLE_CBFLAGS_VOICE,
        }
    }
}

impl PartialChatInfo {
    pub fn from_hashtable(table: &purple::StrHashTable) -> Option<Self> {
        Some(Self {
            group: table.lookup(&GROUP).map(Into::into),
            sn: table.lookup(&SN)?.into(),
            title: table.lookup(&TITLE)?.into(),
        })
    }

    pub fn as_hashtable(&self) -> purple::StrHashTable {
        let mut table = purple::StrHashTable::default();
        table.insert(&SN, &self.sn);
        if let Some(group) = &self.group {
            table.insert(&GROUP, &group);
        }
        table.insert(&TITLE, &self.title);
        table
    }
}

impl ChatInfo {
    pub fn as_partial(&self) -> PartialChatInfo {
        PartialChatInfo {
            sn: self.sn.clone(),
            title: self.title.clone(),
            group: self.group.clone(),
        }
    }

    pub fn need_update(&self, new_version: &ChatInfoVersion) -> bool {
        self.members_version < new_version.members_version
            || self.info_version < new_version.info_version
    }
}

impl From<icq::client::GetChatInfoResponseData> for ChatInfo {
    fn from(info: icq::client::GetChatInfoResponseData) -> Self {
        Self {
            sn: info.sn,
            stamp: Some(info.stamp),
            title: info.name,
            members_version: info.members_version,
            info_version: info.info_version,
            about: info.about,
            members: info
                .members
                .into_iter()
                .map(|m| ChatMember {
                    sn: m.sn,
                    role: MemberRole(m.role),
                    last_seen: m.user_state.last_seen.and_then(|t| match t {
                        0 => None,
                        t => Some(t),
                    }),
                    friendly_name: m.friendly,
                    first_name: m.anketa.first_name,
                    last_name: m.anketa.last_name,
                })
                .collect(),
            ..Default::default()
        }
    }
}

impl From<icq::client::events::HistDlgStateMChatState> for ChatInfoVersion {
    fn from(info: icq::client::events::HistDlgStateMChatState) -> Self {
        Self {
            members_version: info.members_version,
            info_version: info.info_version,
        }
    }
}
