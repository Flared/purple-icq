use serde::{Deserialize, Serialize};
use surf::middleware::HttpClient;

const SEND_CODE_URL: &str = "https://u.icq.net/api/v14/rapi/auth/sendCode";
const LOGIN_WITH_PHONE_NUMBER_URL: &str =
    "https://u.icq.net/api/v14/smsreg/loginWithPhoneNumber.php";
const START_SESSION_URL: &str = "https://u.icq.net/api/v14/wim/aim/startSession?";

#[derive(Debug)]
pub enum Error {
    JsonSerializationError(serde_json::error::Error),
    UrlEncodedSerializationError(serde_urlencoded::ser::Error),
    DeserializationError(serde_json::error::Error),
    RequestError(surf::Error),
}
type Result<T> = std::result::Result<T, Error>;

trait DefaultHeaders {
    fn with_default_headers(self) -> Self;
}

impl<T: HttpClient> DefaultHeaders for surf::Request<T> {
    fn with_default_headers(self) -> Self {
        self.set_header("DNT", "1")
        .set_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.120 Safari/537.36")
        .set_header("Origin", "https://web.icq.com")
        .set_header("Referer", "https://web.icq.com/")
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeBodyParams<'a> {
    pub phone: &'a str,
    pub language: &'a str,
    pub route: &'a str,
    pub dev_id: &'a str,
    pub application: &'a str,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeBody<'a> {
    pub req_id: &'a str,
    pub params: &'a SendCodeBodyParams<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeResponseResults {
    pub code_length: i32,
    pub session_id: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeResponse {
    pub results: SendCodeResponseResults,
}

#[derive(Serialize, Debug)]
pub struct LoginWithPhoneNumberBody<'a> {
    pub msisdn: &'a str,
    pub trans_id: &'a str,
    pub sms_code: &'a str,
    pub locale: &'a str,
    pub k: &'a str,
    pub platform: &'a str,
    pub create_account: &'a str,
    pub client: &'a str,
    pub r: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct WebIcqResponse<T> {
    pub response: WebIcqData<T>,
}

#[derive(Deserialize, Debug)]
pub struct WebIcqData<T> {
    pub data: T,
}

type LoginWithPhoneNumberResponse = WebIcqResponse<LoginWithPhoneNumberResponseData>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginWithPhoneNumberResponseData {
    pub token: LoginWithPhoneNumberResponseToken,
    pub host_time: u32,
    pub session_key: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginWithPhoneNumberResponseToken {
    pub a: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionBody<'a> {
    pub a: &'a str,
    pub ts: u32,
    pub k: &'a str,
    pub view: &'a str,
    pub client_name: &'a str,
    pub language: &'a str,
    pub device_id: &'a str,
    pub session_timeout: u32,
    pub assert_caps: &'a str,
    pub interest_caps: &'a str,
    pub events: &'a str,
    pub include_presence_fields: &'a str,
}

#[derive(Serialize, Debug)]
struct StartSessionFormBody {}

type StartSessionResponse = WebIcqResponse<StartSessionResponseData>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponseData {
    pub aimsid: String,
    pub my_info: StartSessionResponseMyInfo,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponseMyInfo {
    pub aim_id: String,
}

pub async fn send_code(body: &SendCodeBody<'_>) -> Result<SendCodeResponse> {
    post_json(SEND_CODE_URL, body).await
}

pub async fn login_with_phone_number(
    body: &LoginWithPhoneNumberBody<'_>,
) -> Result<LoginWithPhoneNumberResponse> {
    post_form(LOGIN_WITH_PHONE_NUMBER_URL, body).await
}

pub async fn start_session(body: &StartSessionBody<'_>) -> Result<StartSessionResponse> {
    let params = serde_urlencoded::to_string(body).map_err(Error::UrlEncodedSerializationError)?;
    let url = START_SESSION_URL.to_string() + &params;
    post_form(&url, &StartSessionFormBody {}).await
}

async fn post_form<T: serde::Serialize, U: serde::de::DeserializeOwned>(
    url: &str,
    body: &T,
) -> Result<U> {
    log::debug!(
        "POST {} <- {}",
        url,
        serde_urlencoded::to_string(body).unwrap()
    );
    let mut res = surf::post(url)
        .with_default_headers()
        .body_form(&body)
        .map_err(Error::UrlEncodedSerializationError)?
        .await
        .map_err(Error::RequestError)?;
    let body = res.body_string().await;
    log::debug!("POST {} -> {} - {:?}", url, res.status(), body);
    let body = body.map_err(Error::RequestError)?;
    serde_json::from_str(&body).map_err(Error::DeserializationError)
}

async fn post_json<T: serde::Serialize, U: serde::de::DeserializeOwned>(
    url: &str,
    body: &T,
) -> Result<U> {
    log::debug!(
        "POST {} <- {}",
        url,
        serde_json::to_string_pretty(body).unwrap()
    );
    let mut res = surf::post(url)
        .with_default_headers()
        .body_json(&body)
        .map_err(Error::JsonSerializationError)?
        .await
        .map_err(Error::RequestError)?;
    let body = res.body_string().await;
    log::debug!("POST {} -> {} - {:?}", url, res.status(), body);
    let body = body.map_err(Error::RequestError)?;
    serde_json::from_str(&body).map_err(Error::DeserializationError)
}
