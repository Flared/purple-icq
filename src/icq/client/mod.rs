use serde::{Deserialize, Serialize};
use surf::middleware::HttpClient;

pub mod events;
pub mod try_result;

const SEND_CODE_URL: &str = "https://u.icq.net/api/v14/rapi/auth/sendCode";
const LOGIN_WITH_PHONE_NUMBER_URL: &str =
    "https://u.icq.net/api/v14/smsreg/loginWithPhoneNumber.php";
const START_SESSION_URL: &str = "https://u.icq.net/api/v14/wim/aim/startSession?";
const SEND_IM_URL: &str = "https://u.icq.net/api/v14/wim/im/sendIM";
const GET_CHAT_INFO_URL: &str = "https://u.icq.net/api/v14/rapi/getChatInfo";
const JOIN_CHAT_URL: &str = "https://u.icq.net/api/v14/rapi/joinChat";
const FILES_INFO_URL: &str = "https://u.icq.net/api/v14/files/info";

#[derive(Debug)]
pub enum Error {
    JsonSerializationError(serde_json::error::Error),
    UrlEncodedSerializationError(serde_urlencoded::ser::Error),
    DeserializationError(serde_json::error::Error),
    RequestError(surf::Error),
    UrlParseError(url::ParseError),
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RapiBody<'a, T> {
    pub aimsid: &'a str,
    pub req_id: &'a str,
    pub params: T,
}

#[derive(Deserialize, Debug)]
pub struct RapiResponse<T> {
    pub results: T,
}

#[derive(Deserialize, Debug)]
pub struct ResultResponse<T> {
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct EmptyResponse {}

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

type FetchEventsResponse = WebIcqResponse<FetchEventsResponseData>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FetchEventsResponseData {
    pub poll_time: String,
    pub ts: String,
    #[serde(rename = "fetchBaseURL")]
    pub fetch_base_url: String,
    pub fetch_timeout: u32,
    pub time_to_next_fetch: u32,
    pub events: Vec<try_result::TryResult<events::Event>>,
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
    #[serde(rename = "fetchBaseURL")]
    pub fetch_base_url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponseMyInfo {
    pub aim_id: String,
}

pub type GetChatInfoBody<'a> = RapiBody<'a, GetChatInfoBodyParams<'a>>;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChatInfoBodyParams<'a> {
    pub member_limit: u32,
    pub stamp: &'a str,
}

type GetChatInfoResponse = RapiResponse<GetChatInfoResponseData>;

#[derive(Serialize, Debug)]
pub struct StampBodyParams<'a> {
    pub stamp: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChatInfoResponseData {
    pub name: String,
    pub stamp: String,
    //pub create_time: usize,
    //pub public: bool,
    //pub live: bool,
    //pub controlled: bool,
    //pub members_count: usize,
    //pub admins_count: usize,
    //pub default_role: String,
    //pub regions: String,
    pub sn: String,
    //pub abuse_reports_current_count: usize,
    pub persons: Vec<ChatInfoResponsePerson>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatInfoResponsePerson {
    pub sn: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub friendly: Option<String>,
}

pub type JoinChatBody<'a> = RapiBody<'a, StampBodyParams<'a>>;

pub type JoinChatResponse = RapiResponse<EmptyResponse>;

#[derive(Serialize, Debug)]
pub struct SendIMBody<'a> {
    pub t: &'a str,
    pub r: &'a str,
    pub mentions: &'a str,
    pub message: &'a str,
    pub f: &'a str,
    pub aimsid: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendIMResponseData {
    pub msg_id: String,
    pub hist_msg_id: u64,
    pub before_hist_msg_id: u64,
    pub ts: u32,
    pub state: String,
}

pub type SendIMResponse = WebIcqResponse<SendIMResponseData>;

#[derive(Serialize, Debug)]
pub struct FilesInfoBody<'a> {
    pub aimsid: &'a str,
    pub previews: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilesInfoResponseData {
    pub info: FilesInfoResponseDataInfo,
    pub previews: FilesInfoResponseDataPreviews,
}

#[derive(Deserialize, Debug)]
pub struct FilesInfoResponseDataPreviews {
    #[serde(rename = "192")]
    pub x192: String,
}

#[derive(Deserialize, Debug)]
pub struct FilesInfoResponseDataInfo {
    pub file_size: usize,
    pub file_name: String,
    pub md5: String,
    pub dlink: String,
    pub mime: String,
}

pub type FilesInfoResponse = ResultResponse<FilesInfoResponseData>;

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

pub async fn send_im(body: &SendIMBody<'_>) -> Result<SendIMResponse> {
    post_form(&SEND_IM_URL, body).await
}

pub async fn fetch_events(fetch_base_url: &str) -> Result<FetchEventsResponse> {
    let url = url::Url::parse_with_params(fetch_base_url, &[("timeout", "30000")])
        .map_err(Error::UrlParseError)?;
    get_json(&url.to_string()).await
}

pub async fn get_chat_info(body: &GetChatInfoBody<'_>) -> Result<GetChatInfoResponse> {
    post_json(GET_CHAT_INFO_URL, body).await
}

pub async fn join_chat(body: &JoinChatBody<'_>) -> Result<JoinChatResponse> {
    post_json(JOIN_CHAT_URL, body).await
}

pub async fn files_info(file_id: &str, body: &FilesInfoBody<'_>) -> Result<FilesInfoResponse> {
    let params = serde_urlencoded::to_string(body).map_err(Error::UrlEncodedSerializationError)?;
    let url = format!("{}/{}?{}", FILES_INFO_URL, file_id, params);
    get_json(&url).await
}

async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    log::debug!("GET {}", url);

    let mut res = surf::get(url)
        .with_default_headers()
        .await
        .map_err(Error::RequestError)?;

    let body = res.body_string().await;
    log::debug!("GET {} -> {} - {:?}", url, res.status(), body);
    let body = body.map_err(Error::RequestError)?;

    serde_json::from_str(&body).map_err(Error::DeserializationError)
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
