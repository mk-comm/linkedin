use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::conversation::Conversation;
use crate::structs::entry::EntryRegular;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::actions::scrap_messages::scrap_message;

pub async fn scrap(entry: EntryRegular) -> Result<(), CustomError> {
    let api_key = entry.user_id.clone();
    let regular = entry.regular.clone();

    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: None,
        headless: true,
    };

    let browser = start_browser(browser_info).await?;

    wait(3, 11);
    /*
    let messaging_button = browser
        .page
        .query_selector("a.global-nav__primary-link:has-text('Messaging')")
        .await
        .unwrap();
    match messaging_button {
        Some(messaging_button) => {
            messaging_button.click_builder().click().await.unwrap();
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound("Messaging button not found".to_string()));
        }
    }
    */

    wait(1, 12);

    let build = browser.page.goto_builder("https://www.linkedin.com/messaging/thread/2-NjhlODRmMzUtZTZkYi00MDNjLThmNzMtMDJlNm44RmMjU1NDY2XzAxMw==/");
    wait(1, 8);
    let go_to = build.goto().await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.page.goto_builder("https://www.linkedin.com/messaging/thread/2-NjhlODRmMzUNm44RmMjU1NDY2XzAxMw==/")
            .goto().await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(1, 3);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?;
                return Err(CustomError::ButtonNotFound(
                    "Conversation page is not loading".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            //println!("retrying to load page")
        }
        wait(6, 9);
    }
    wait(6, 9);
    let focused = browser
        .page
        .query_selector("div.artdeco-tabs.artdeco-tabs--size-t-40.artdeco-tabs--centered.ember-view.msg-focused-inbox-tabs")
        .await?;

    let focused_inbox = match focused {
        Some(_) => true,
        None => false,
    };

    let conversation_list = match browser
        .page
        .query_selector(
            "ul[class='list-style-none msg-conversations-container__conversations-list']",
        )
        .await?
    {
        Some(conversation_list) => conversation_list,
        None => {
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Conversation list not found".to_string(),
            ));
        }
    };

    let conversations =
        scrap_conversation_to_list(&conversation_list.inner_html().await?, &api_key, regular);

    for conversation in conversations.values() {
        scrap_message(conversation, &browser.page, focused_inbox, &browser).await?;
    }

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
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
        if convo.select(&thread_url_selector).next() == None {
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
