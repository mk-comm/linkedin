use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;
#[derive(Debug)]
pub enum CustomError {
    PlaywrightError(Arc<playwright::Error>),
    ButtonNotFound(String),
    ReqwestError(reqwest::Error),
    SessionCookieExpired,
    RecruiterSessionCookieExpired,
    ProxyNotWorking,
    EmailNeeded,
    ConnectionLimit,
    //ProfileNotFound,
    AnyhowError(anyhow::Error),
    ChronoError(chrono::ParseError),
    SerdeJsonError(serde_json::Error),
    BoxedError(Box<dyn StdError + Send + Sync>),
}
unsafe impl Send for CustomError {}
unsafe impl Sync for CustomError {}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::PlaywrightError(e) => write!(f, "{}", e),
            CustomError::ButtonNotFound(e) => write!(f, "{}", e),
            CustomError::SessionCookieExpired => write!(f, "Session cookie expired"),
            CustomError::RecruiterSessionCookieExpired => {
                write!(f, "Recruiter Session cookie expired")
            }
            //CustomError::ProfileNotFound => write!(f, "Profile not found"),
            CustomError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            CustomError::EmailNeeded => write!(f, "Email needed"),
            CustomError::ConnectionLimit => write!(f, "Connection limit"),
            CustomError::ProxyNotWorking => write!(f, "Proxy not working"),
            CustomError::ReqwestError(e) => write!(f, "{}", e),
            //CustomError::ActixError(e) => write!(f, "{}", e),
            CustomError::AnyhowError(e) => write!(f, "{}", e),
            CustomError::SerdeJsonError(e) => write!(f, "{}", e),
            CustomError::ChronoError(e) => write!(f, "{}", e),
            //CustomError::IoError(e) => write!(f, "{}", e),
            CustomError::BoxedError(err) => write!(f, "Boxed error: {}", err),
        }
    }
}
impl StdError for CustomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CustomError::PlaywrightError(err) => Some(err.as_ref()),
            CustomError::ReqwestError(err) => Some(err),
            CustomError::AnyhowError(err) => Some(err.root_cause()),
            CustomError::ChronoError(err) => Some(err),
            CustomError::SerdeJsonError(err) => Some(err),
            _ => None,
        }
    }
}
impl From<Box<dyn StdError + Send + Sync>> for CustomError {
    fn from(error: Box<dyn StdError + Send + Sync>) -> Self {
        CustomError::BoxedError(error)
    }
}
impl From<Arc<playwright::Error>> for CustomError {
    fn from(err: Arc<playwright::Error>) -> CustomError {
        CustomError::PlaywrightError(err)
    }
}

impl From<playwright::Error> for CustomError {
    fn from(err: playwright::Error) -> CustomError {
        CustomError::PlaywrightError(err.into())
    }
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> CustomError {
        CustomError::ReqwestError(err)
    }
}

impl From<anyhow::Error> for CustomError {
    fn from(err: anyhow::Error) -> CustomError {
        CustomError::AnyhowError(err)
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(err: serde_json::Error) -> Self {
        CustomError::SerdeJsonError(err)
    }
}

impl From<chrono::ParseError> for CustomError {
    fn from(err: chrono::ParseError) -> Self {
        CustomError::ChronoError(err)
    }
}
