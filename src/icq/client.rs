use reqwest::header;
use serde;
use serde::Serialize;

const SEND_CODE_URL: &'static str = "https://u.icq.net/api/v14/rapi/auth/sendCode";

lazy_static::lazy_static! {
    static ref HEADERS: header::HeaderMap = {
        h = header::HeaderMap::new();
        h.insert(header::DNT, "1");
        h.insert(header::USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.120 Safari/537.36");
        h.insert(header::ORIGIN, "https://web.icq.com");
        h.insert(header::REFERER, "https://web.icq.com/");
        h
    };
}

type Result<T> = Result<T, reqwest::Error>;

#[derive(Serialize, Debug)]
struct SendCodeBodyParams {
    phone: String,
    language: String,
    route: String,
    #[serde(rename = "devId")]
    dev_id: String,
    application: String,
}

#[derive(Serialize, Debug)]
struct SendCodeBody {
    reqId: String,
    params: SendDataBodyParams,
}

async fn send_code(body: &SendCodeBody) -> Result<SendCodeResponse> {
    reqwest::Client::new()
        .post(SEND_CODE_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await
}
