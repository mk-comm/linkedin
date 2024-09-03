use playwright::api::Page;
use serde_json::json;
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;
use thirtyfour::error::WebDriverError;
use tracing::{error, info};
#[derive(Debug)]
pub enum CustomError {
    PlaywrightError(Arc<playwright::Error>),
    ButtonNotFound(String),
    ErrorWithString(String),
    ReqwestError(reqwest::Error),
    SessionCookieExpired,
    RecruiterSessionCookieExpired,
    ProxyNotWorking,
    WebDriverError(WebDriverError),
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
            CustomError::ErrorWithString(e) => write!(f, "{}", e),
            CustomError::SessionCookieExpired => write!(f, "Session cookie expired"),
            CustomError::RecruiterSessionCookieExpired => {
                write!(f, "Recruiter Session cookie expired")
            }
            CustomError::WebDriverError(err) => write!(f, "WebDriverError: {}", err),
            //CustomError::ProfileNotFound => write!(f, "Profile not found"),
            CustomError::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            CustomError::EmailNeeded => write!(f, "Email needed"),
            CustomError::ConnectionLimit => write!(f, "Connection limit"),
            CustomError::ProxyNotWorking => write!(f, "Proxy not working"),
            CustomError::AnyhowError(e) => write!(f, "{}", e),
            CustomError::SerdeJsonError(e) => write!(f, "{}", e),
            CustomError::ChronoError(e) => write!(f, "{}", e),
            CustomError::BoxedError(err) => write!(f, "Boxed error: {}", err),
        }
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

pub async fn error_and_screenshot(
    page: &Page,
    error: &str,
    function: &str,
    user_id: &str,
) -> CustomError {
    let screenshot = page.screenshot_builder().screenshot().await;
    send_screenshot(screenshot.unwrap(), &user_id, &error, &function).await;
    CustomError::ErrorWithString(error.to_string())
}

pub async fn send_screenshot(
    screenshot: Vec<u8>,
    api_key: &str,
    variant: &str,
    message_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let screenshot = base64::encode(&screenshot);
    let send_json = json!({
        "screenshot": screenshot,
        "api_key":  api_key,
        "variant": variant,
        "message_id": message_id,
    });
    const TARGET_URL: &str = "
       https://tribexyz.bubbleapps.io/version-test/api/1.1/wf/sequence_status_screenshot/";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(TARGET_URL).json(&send_json).send().await;
    match response {
        Ok(_) => info!("{}/Ok, {} was done", variant, api_key),
        Err(error) => {
            error!(error = ?error, "{}/Error {} returned error {}", variant, api_key, error);
        }
    }

    Ok(())
}
