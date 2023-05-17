use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub enum CustomError {
   PlaywrightError(Arc<playwright::Error>),
   ButtonNotFound(String),
   SessionCookieExpired,
   ProxyNotWorking,
   EmailNeeded,
   ConnectionLimit,
}

impl fmt::Display for CustomError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
         CustomError::PlaywrightError(e) => write!(f, "{}", e),
         CustomError::ButtonNotFound(e) => write!(f, "{}", e),
         CustomError::SessionCookieExpired => write!(f, "Session cookie expired"),
         CustomError::EmailNeeded => write!(f, "Email needed"),
         CustomError::ConnectionLimit => write!(f, "Connection limit"),
         CustomError::ProxyNotWorking => write!(f, "Proxy not working"),
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