use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::conversation::Conversation;
use crate::structs::entry::Entry;
use scraper::{Html, Selector};
use std::collections::HashMap;
use crate::structs::error::CustomError;

use crate::actions::scrap_messages::scrap_message;

pub async fn scrap(entry: Entry) -> Result<(), CustomError> {
    let api_key = entry.user_id.clone();
    let regular = entry.regular.clone();
    let browser = start_browser(entry).await?;

    wait(3, 7);

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
            return Err(playwright::Error::NotObject.into());
        }
    }

    wait(5, 10);

    let focused = browser
        .page
        .query_selector("div.artdeco-tabs.artdeco-tabs--size-t-40.artdeco-tabs--centered.ember-view.msg-focused-inbox-tabs")
        .await?;

    let focused_inbox = match focused {
        Some(_) => {
            true
        }
        None => {
            false
        }
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
            return Err(playwright::Error::ObjectNotFound.into());
        }
    };

    let document = Html::parse_document(&conversation_list.inner_html().await.unwrap()); // parse html
                                                                                         //selectors (which part of html to parse to get the specific data)
    let conversation_selector = Selector::parse("li.msg-conversation-listitem").unwrap();

    let participant_name_selector =
        Selector::parse("h3.msg-conversation-listitem__participant-names").unwrap();
    let timestamp_selector = Selector::parse("time.msg-conversation-listitem__time-stamp").unwrap();
    let thread_url_selector = Selector::parse("a.msg-conversation-listitem__link").unwrap();
    let unread_selector = Selector::parse(".msg-conversation-card__unread-count").unwrap();

    let mut conversations = HashMap::new(); // hashmap to store conversations

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
            api_key: api_key.clone(),
            enable_ai: regular,
        };

        conversations.insert(id, conversation);
    }

    for conversation in conversations.values() {
        scrap_message(conversation, &browser.page, focused_inbox, &browser).await?;
    }
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}
