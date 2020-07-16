use serde::{Deserialize, Serialize};
use surf::middleware::HttpClient;

const SEND_CODE_URL: &str = "https://u.icq.net/api/v14/rapi/auth/sendCode";
const LOGIN_WITH_PHONE_NUMBER_URL: &str =
    "https://u.icq.net/api/v14/smsreg/loginWithPhoneNumber.php";

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
pub struct LoginWithPhoneNumberResponse {
    pub response: LoginWithPhoneNumberResponseResponse,
}

#[derive(Deserialize, Debug)]
pub struct LoginWithPhoneNumberResponseResponse {
    pub data: LoginWithPhoneNumberResponseData,
}

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

pub async fn send_code(body: &SendCodeBody<'_>) -> Result<SendCodeResponse> {
    json_api(SEND_CODE_URL, body).await
}

pub async fn login_with_phone_number(
    body: &LoginWithPhoneNumberBody<'_>,
) -> Result<LoginWithPhoneNumberResponse> {
    form_api(LOGIN_WITH_PHONE_NUMBER_URL, body).await
}

async fn form_api<T: serde::Serialize, U: serde::de::DeserializeOwned>(
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

async fn json_api<T: serde::Serialize, U: serde::de::DeserializeOwned>(
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
