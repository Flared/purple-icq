use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Try<T> {
    Ok(T),
    Unknown(serde_json::Value),
}

#[derive(Deserialize, Debug)]
#[serde(from = "Try<T>")]
pub struct TryResult<T>(pub std::result::Result<T, serde_json::Value>);

impl<T> From<Try<T>> for TryResult<T> {
    fn from(other: Try<T>) -> TryResult<T> {
        match other {
            Try::Ok(v) => TryResult(Ok(v)),
            Try::Unknown(v) => TryResult(Err(v)),
        }
    }
}

impl<T> std::ops::Deref for TryResult<T> {
    type Target = std::result::Result<T, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
