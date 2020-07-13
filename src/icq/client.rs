use reqwest::header::{self, HeaderMap, HeaderValue};
use serde;
use serde::{Deserialize, Serialize};

const SEND_CODE_URL: &'static str = "https://u.icq.net/api/v14/rapi/auth/sendCode";

lazy_static::lazy_static! {
    static ref HEADERS: HeaderMap = {
        let mut h = HeaderMap::new();
        h.insert(header::DNT, HeaderValue::from_static("1"));
        h.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.120 Safari/537.36"));
        h.insert(header::ORIGIN, HeaderValue::from_static("https://web.icq.com"));
        h.insert(header::REFERER, HeaderValue::from_static("https://web.icq.com/"));
        h
    };
}

type Result<T> = std::result::Result<T, reqwest::Error>;

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

pub async fn send_code(body: &SendCodeBody<'_>) -> Result<SendCodeResponse> {
    reqwest::Client::new()
        .post(SEND_CODE_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await
}
