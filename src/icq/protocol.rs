use super::client;

#[derive(Debug)]
enum Error {
    ApiError(reqwest::Error),
}

type Result<T> = Result<T, Error>;

pub struct RegisteredAccountInfo {}

pub async fn register<F, Fut>(
    phone_number: String,
    code_validator: F,
) -> Result<RegisteredAccountInfo>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = String>,
{
    client::send_code().await.map_err(Error::ApiError)?;
}
