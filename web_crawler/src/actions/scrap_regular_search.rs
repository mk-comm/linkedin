use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapSearchRegular;
use crate::structs::error::CustomError;
use anyhow::Ok;
use scraper::{Html, Selector};
use serde_json::json;






pub async fn send_message(entry: EntryScrapSearchRegular) -> Result<(), CustomError> {
   let browser_info = BrowserInit {
      ip: entry.ip,
      username: entry.username,
      password: entry.password,
      user_agent: entry.user_agent,
      session_cookie: entry.session_cookie,
      user_id: entry.user_id,
      recruiter_session_cookie: None,
  };

  let browser = start_browser(browser_info).await?;

  Ok(())
}