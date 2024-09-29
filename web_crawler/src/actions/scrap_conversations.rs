use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::conversation::Conversation;
use crate::structs::entry::EntryRegular;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};

use std::collections::HashMap;
use thirtyfour::{By, WebDriver};

use crate::actions::scrap_messages::scrap_message;

pub async fn scrap(entry: EntryRegular) -> Result<String, CustomError> {
    let api_key = entry.user_id.clone();
    let regular = entry.regular;
    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        user_id: entry.user_id,
        session_cookie: entry.cookies.session_cookie,
        recruiter_session_cookie: entry.cookies.recruiter_session_cookie,
        bscookie: entry.cookies.bscookie,
        bcookie: entry.cookies.bcookie,
        fcookie: entry.cookies.fcookie,
        fidcookie: entry.cookies.fidcookie,
        jsessionid: entry.cookies.jsessionid,
    };
    let browser = init_browser(&browser_info).await?;
    //wait(10000, 10000);
    let result = start_scrap(&browser, regular, api_key.as_str()).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                text.as_str(),
                &api_key,
                "Scrap Inmails",
            )
            .await?;
            browser.quit().await?;
            return Ok(text);
        }
        Err(error) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                error.to_string().as_str(),
                &api_key,
                "Scrap Inmails",
            )
            .await?;
            return Err(error);
        }
    }
}
pub async fn start_scrap(
    browser: &WebDriver,
    regular: bool,
    api_key: &str,
) -> Result<String, CustomError> {
    wait(1, 12);
    const CONVERSATION_URL:&str = "https://www.linkedin.com/messaging/thread/2-NjhlODRmMzUtZTZkYi00MDNjLThmNzMtMDJlNm44RmMjU1NDY2XzAxMw==/";
    let go_to = browser.goto(CONVERSATION_URL).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(CONVERSATION_URL).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                return Err(CustomError::ButtonNotFound(
                    "Conversation page is not loading/Scrap conversation".to_string(),
                ));
            }
            x += 1;
        }
        wait(1, 3);
    }
    wait(15, 17);

    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const FOCUSED:&str = "div.artdeco-tabs.artdeco-tabs--size-t-40.artdeco-tabs--centered.ember-view.msg-focused-inbox-tabs";

    let focused = browser.find(By::Css(FOCUSED)).await;

    let focused_inbox = match focused {
        Ok(_) => true,
        Err(_) => false,
    };
    const CONVERSATION_LIST: &str =
        "ul[class='list-style-none msg-conversations-container__conversations-list']";
    let conversation_list = match browser.find(By::Css(CONVERSATION_LIST)).await {
        Ok(list) => list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Conversation list not found".to_string(),
            ));
        }
    };

    let conversations =
        scrap_conversation_to_list(&conversation_list.inner_html().await?, &api_key, regular);

    for conversation in conversations.values() {
        scrap_message(conversation, focused_inbox, &browser).await?;
    }

    Ok("Messages were scrapped successfully".to_string())
}

fn scrap_conversation_to_list(
    html: &str,
    api_key: &str,
    regular: bool,
) -> HashMap<String, Conversation> {
    let mut conversations = HashMap::new(); // hashmap to store conversations

    let document = Html::parse_document(html); // parse html

    let conversation_selector = Selector::parse("li.msg-conversation-listitem").unwrap();

    let participant_name_selector =
        Selector::parse("h3.msg-conversation-listitem__participant-names").unwrap();
    let timestamp_selector = Selector::parse("time.msg-conversation-listitem__time-stamp").unwrap();
    let thread_url_selector = Selector::parse("a.msg-conversation-listitem__link").unwrap();
    let unread_selector = Selector::parse(".msg-conversation-card__unread-count").unwrap();

    for convo in document.select(&conversation_selector) {
        let id = convo.value().attr("id").unwrap().to_string();

        //once conversation thread url is not found, break the loop, that means it was the last convo
        if convo.select(&thread_url_selector).next().is_none() {
            break;
        }
        let thread_url = convo
            .select(&thread_url_selector)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .to_string();

        let candidate_name = convo
            .select(&participant_name_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let timestamp = convo
            .select(&timestamp_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        let unread = match convo.select(&unread_selector).next() {
            Some(_) => true,
            None => false,
        };

        let conversation = Conversation {
            id: id.clone(),
            thread_url,
            candidate_name,
            timestamp,
            unread,
            api_key: api_key.to_string(),
            enable_ai: regular,
        };

        conversations.insert(id, conversation);
    }

    conversations
}
