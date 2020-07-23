use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub seq_num: u32,

    #[serde(flatten)]
    pub event_data: EventData,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "eventData")]
#[serde(rename_all = "camelCase")]
pub enum EventData {
    HistDlgState(HistDlgStateData),

    #[serde(rename = "buddylist")]
    BuddyList(BuddyListData),

    PermitDeny(PermitDenyData),

    MyInfo(MyInfoData),

    Presence(PresenceData),

    GalleryNotify(GalleryNotifyData),
}

// Event: BuddyList

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuddyListData {
    pub groups: Vec<BuddyListGroup>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuddyListGroup {
    // Example:
    // {
    //      'name': 'General',
    //      'id': 1,
    //      'buddies': [ ... ],
    // },
    // {
    //      'name': 'Temporarily',
    //      'id': 1,
    //      'buddies': [ ... ],
    // },
    // {
    //      'name': 'Conferences',
    //      'id': 1,
    //      'buddies': [ ... ],
    // }
    name: String,
    id: u32,
    pub buddies: Vec<Buddy>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Buddy {
    // Example (from General)
    // {
    //      'nick': 'icq.com',
    //      'aimId': '11111',
    //      'displayId': '11111',
    //      'friendly': 'ICQ Official',
    //      'state': 'online',
    //      'userType': 'icq',
    //      'official': 1,
    //      'lastseen': 0
    // }
    //
    // Example (From Temporarily)
    // {
    //      'aimId': '111111111',
    //      'displayId': '111111111',
    //      'friendly': 'Alex Viau',
    //      'state': 'offline',
    //      'userType': 'icq',
    //      'autoAddition': 'autoAccepted',
    //      'lastseen': 1111111111
    // }
    //
    // Example (From Conferences)
    // {
    //      'aimId': '111111111@chat.agent',
    //      'displayId': '111111111@chat.agent',
    //      'friendly': 'test conference please ignore',
    //      'state': 'online',
    //      'userType': 'chat',
    //      'chatType': 'group'
    // }
    nick: Option<String>,
    aim_id: String,
    display_id: String,
    friendly: Option<String>,
    state: String,
    pub user_type: UserType,
    official: Option<u32>,
    chat_type: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    ICQ,
    Chat,
    #[serde(other)]
    Unknown,
}

// Event: PermitDeny

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PermitDenyData {}

// Event: MyInfo

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MyInfoData {
    // Example:
    // {
    //     'aimId': '111111111',
    //     'displayId': '111111111',
    //     'friendly': 'hello hello',
    //     'state': 'online',
    //     'userType': 'icq',
    //     'attachedPhoneNumber': '13111111111',
    //     'globalFlags': '32'
    // }
    aim_id: String,
    display_id: String,
    friendly: Option<String>,
    state: String,
    user_type: String,
    attached_phone_number: String,
    global_flags: String,
}

// Event: Presence

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresenceData {
    // Example:
    // {
    //      "nick": "111111111",
    //      "aimId": "111111111",
    //      "displayId": "111111111",
    //      "friendly": "Alex Viau",
    //      "state": "offline",
    //      "userType": "icq",
    //      "statusTime": 17090335,
    //      "statusMsg": "I\'m not here right now",
    //      "autoAddition": "autoAdded",
    //      "lastseen": 1111111111,
    // }
    aim_id: String,
    display_id: String,
    friendly: Option<String>,
    state: String,
    user_type: String,
    status_time: u32,
    status_msg: String,
    auto_addition: String,
    lastseen: u32,
}

// Event: GalleryNotify

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GalleryNotifyData {}

// Event: HistDlgState

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistDlgStateData {
    sn: String,
    starting: Option<bool>,
    last_msg_id: String,
    last_read_mention: Option<String>,
    patch_version: String,
    unread_cnt: u32,
}

// TODO: unused for now
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistDlgStateMessage {
    msg_id: String,
    time: u32,
    locale: String,
    text: String,
    //TODO: parts
    media_type: String,
    chat: HistDlgStateMessageChat,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HistDlgStateMessageChat {
    sender: String,
    name: String,
    live: bool,
}
