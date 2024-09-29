use scraper::error::SelectorErrorKind;
use std::error::Error as StdError;
use std::fmt;
use thirtyfour::error::WebDriverError;
#[derive(Debug)]
pub enum CustomError {
    ButtonNotFound(String),
    SelectorError(String),
    ReqwestError(reqwest::Error),
    SessionCookieExpired,
    RecruiterSessionCookieExpired,
    WebDriverError(WebDriverError),
    ConnectionLimit,
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
            CustomError::ButtonNotFound(e) => write!(f, "{}", e),
            CustomError::SelectorError(e) => write!(f, "Selector error: {:?}", e),
            CustomError::SessionCookieExpired => write!(f, "Session cookie expired"),
            CustomError::RecruiterSessionCookieExpired => {
                write!(f, "Recruiter Session cookie expired")
            }
            CustomError::WebDriverError(err) => write!(f, "WebDriverError: {}", err),
            //CustomError::ProfileNotFound => write!(f, "Profile not found"),
            CustomError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            //CustomError::EmailNeeded => write!(f, "Email needed"),
            CustomError::ConnectionLimit => write!(f, "Connection limit"),
            //CustomError::ProxyNotWorking => write!(f, "Proxy not working"),
            CustomError::AnyhowError(e) => write!(f, "{}", e),
            CustomError::SerdeJsonError(e) => write!(f, "{}", e),
            CustomError::ChronoError(e) => write!(f, "{}", e),
            CustomError::BoxedError(err) => write!(f, "Boxed error: {}", err),
        }
    }
}
impl From<SelectorErrorKind<'_>> for CustomError {
    fn from(err: SelectorErrorKind<'_>) -> Self {
        CustomError::SelectorError(format!("{:?}", err))
    }
}
impl From<WebDriverError> for CustomError {
    fn from(err: WebDriverError) -> CustomError {
        CustomError::WebDriverError(err.into())
    }
}
impl StdError for CustomError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
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
