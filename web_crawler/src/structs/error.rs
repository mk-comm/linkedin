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
}

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
            CustomError::EmailNeeded => write!(f, "Email needed"),
            CustomError::ConnectionLimit => write!(f, "Connection limit"),
            CustomError::ProxyNotWorking => write!(f, "Proxy not working"),
            CustomError::ReqwestError(e) => write!(f, "{}", e),
            //CustomError::ActixError(e) => write!(f, "{}", e),
            CustomError::AnyhowError(e) => write!(f, "{}", e),
            CustomError::SerdeJsonError(e) => write!(f, "{}", e),
            CustomError::ChronoError(e) => write!(f, "{}", e),
            //CustomError::IoError(e) => write!(f, "{}", e),
        }
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
