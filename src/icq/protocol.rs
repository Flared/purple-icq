use super::client;
use super::client::events::Event;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::SystemTime;
use uuid::Uuid;

const LANGUAGE: &str = "en-US";
const KEY: &str = "ic1rtwz1s1Hj1O0r";
const LOCALE: &str = "en-US";
const CAPS: &str = "094613584C7F11D18222444553540000,0946135C4C7F11D18222444553540000,0946135b4c7f11d18222444553540000,0946135E4C7F11D18222444553540000,AABC2A1AF270424598B36993C6231952,1f99494e76cbc880215d6aeab8e42268";
const EVENTS: &str = "myInfo,presence,buddylist,typing,hiddenChat,hist,mchat,sentIM,imState,dataIM,offlineIM,userAddedToBuddyList,service,lifestream,apps,permitDeny,diff,webrtcMsg";
const PRESENCE_FIELDS: &str = "aimId,displayId,friendly,friendlyName,state,userType,statusMsg,statusTime,lastseen,ssl,mute,abContactName,abPhoneNumber,abPhones,official,quiet,autoAddition,largeIconId,nick,userState";

#[derive(Debug)]
pub enum Error {
    ApiError(client::Error),
    MissingCode,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub registration_data: RegistrationData,
    pub aim_id: String,
    pub aim_sid: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegistrationData {
    pub session_id: String,
    pub session_key: String,
    pub token: String,
    pub host_time: u32,
}

impl RegistrationData {
    pub const SESSION_ID_SETTING_KEY: &'static str = "session_id";
    pub const SESSION_KEY_SETTING_KEY: &'static str = "session_key";
    pub const TOKEN_SETTING_KEY: &'static str = "token";
    pub const HOST_TIME_SETTING_KEY: &'static str = "host_time";
}

pub async fn register<F, Fut>(phone_number: &str, code_validator: F) -> Result<RegistrationData>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let send_code_body = client::SendCodeBody {
        req_id: &request_id(),
        params: &client::SendCodeBodyParams {
            phone: &phone_number,
            language: LANGUAGE,
            route: "sms",
            dev_id: KEY,
            application: "icq",
        },
    };
    let code_response = client::send_code(&send_code_body)
        .await
        .map_err(Error::ApiError)?;
    log::info!("SendCode response: {:?}", code_response);

    let code = code_validator().await.ok_or(Error::MissingCode)?;

    let login_with_phone_number_body = client::LoginWithPhoneNumberBody {
        msisdn: &phone_number,
        trans_id: &code_response.results.session_id,
        sms_code: &code,
        locale: LOCALE,
        k: KEY,
        platform: "web",
        create_account: "1",
        client: "icq",
        r: &random_id(),
    };
    let login_response = client::login_with_phone_number(&login_with_phone_number_body)
        .await
        .map_err(Error::ApiError)?;
    log::info!("Login response: {:?}", login_response);
    Ok(RegistrationData {
        session_id: code_response.results.session_id,
        session_key: login_response.response.data.session_key,
        host_time: login_response.response.data.host_time,
        token: login_response.response.data.token.a,
    })
}

pub async fn start_session(registration_data: &RegistrationData) -> Result<SessionInfo> {
    let start_session_body = client::StartSessionBody {
        a: &registration_data.token,
        ts: timestamp(),
        k: KEY,
        view: "online",
        client_name: "webicq",
        language: LANGUAGE,
        device_id: &device_id(),
        session_timeout: 2_592_000,
        assert_caps: CAPS,
        interest_caps: "",
        events: EVENTS,
        include_presence_fields: PRESENCE_FIELDS,
    };
    let start_session_response = client::start_session(&start_session_body)
        .await
        .map_err(Error::ApiError)?;
    Ok(SessionInfo {
        registration_data: registration_data.clone(),
        aim_id: start_session_response.response.data.my_info.aim_id,
        aim_sid: start_session_response.response.data.aimsid,
    })
}

pub async fn fetch_events(session_info: &SessionInfo, seq_num: u32) -> Result<Vec<Event>> {
    let fetch_events_body = client::FetchEventsBody {
        aimsid: &session_info.aim_sid,
        bg: 1,
        hidden: 1,
        rnd: "1.1",
        timeout: 30000,
        seq_num,
        supported_suggest_types: "sticker-smartreply,text-smartreply",
    };

    let fetch_events_response = client::fetch_events(&fetch_events_body)
        .await
        .map_err(Error::ApiError)?;

    Ok(fetch_events_response.response.data.events)
}

fn request_id() -> String {
    format!("{}-{}", random_id(), timestamp())
}

fn random_id() -> String {
    let random_id = rand::thread_rng().gen_range(10_000, 100_000);
    random_id.to_string()
}

fn timestamp() -> u32 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

fn device_id() -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, random_id().as_bytes())
        .to_hyphenated()
        .to_string()
}
