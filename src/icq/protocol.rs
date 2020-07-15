use super::client;
use rand::Rng;
use std::future::Future;
use std::time::SystemTime;

const LANGUAGE: &'static str = "en-US";
const KEY: &'static str = "ic1rtwz1s1Hj1O0r";

#[derive(Debug)]
pub enum Error {
    ApiError(reqwest::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct RegisteredAccountInfo {}

pub async fn register<'a, F, Fut>(
    phone_number: &'a str,
    code_validator: F,
) -> Result<RegisteredAccountInfo>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let _send_code_body = client::SendCodeBody {
        req_id: &request_id(),
        params: &client::SendCodeBodyParams {
            phone: &phone_number,
            language: LANGUAGE,
            route: "sms",
            dev_id: KEY,
            application: "icq",
        },
    };
    //let _code_response = client::send_code(&send_code_body)
    //    .await
    //    .map_err(Error::ApiError)?;
    let _code = code_validator().await;
    Ok(RegisteredAccountInfo {})
}

fn request_id() -> String {
    let random_id = rand::thread_rng().gen_range(10000, 100000);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}-{}", random_id, timestamp)
}
