use super::client;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::SystemTime;

const LANGUAGE: &str = "en-US";
const KEY: &str = "ic1rtwz1s1Hj1O0r";
const LOCALE: &str = "en-US";

#[derive(Debug)]
pub enum Error {
    ApiError(client::Error),
    MissingCode,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct SessionInfo {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisteredAccountInfo {
    pub session_id: String,
    pub session_key: String,
    pub token: String,
    pub host_time: u32,
}

pub async fn register<F, Fut>(
    phone_number: &str,
    code_validator: F,
) -> Result<RegisteredAccountInfo>
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
    Ok(RegisteredAccountInfo {
        session_id: code_response.results.session_id,
        session_key: login_response.response.data.session_key,
        host_time: login_response.response.data.host_time,
        token: login_response.response.data.token.a,
    })
}

pub fn start_session(_registered_account_info: &RegisteredAccountInfo) -> Result<SessionInfo> {
    Ok(SessionInfo {})
}

fn request_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{}", random_id(), timestamp)
}

fn random_id() -> String {
    let random_id = rand::thread_rng().gen_range(10_000, 100_000);
    random_id.to_string()
}
