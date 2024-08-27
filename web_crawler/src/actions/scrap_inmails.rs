use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::{BrowserConfig, BrowserInit};
use crate::structs::entry::EntryRecruiter;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use crate::structs::inmail_conversation::InmailConversation;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

pub async fn scrap_inmails(entry: EntryRecruiter) -> Result<(), CustomError> {
    let recruiter = entry.recruiter;
    let api_key = entry.user_id.clone();
    let stage_interested = entry.recruiter_stage_interested.clone();
    let stage_not_interested = entry.recruiter_stage_not_interested.clone();

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

    let browser = start_browser(browser_info).await?;
    wait(7, 10); // random delay
                 // go to candidate page
    browser
        .page
        .goto_builder("https://www.linkedin.com/talent/inbox/0/main/id/2-MTBiZjJhZTMtNTNlNi00NDRjLddWJmZGQtYTg5MTk4ZjA5MWExXzAxMg==")
        .goto()
        .await?;

    wait(7, 12);

    let nav_bar = browser
        .page
        .query_selector("div[class='global-nav__right']")
        .await?;

    match &nav_bar {
        Some(_) => {}
        None => {
            wait(1, 3);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::RecruiterSessionCookieExpired); // if error when session cookie expired
        }
    }

    let conversation_list = match browser
        .page
        .query_selector("div.thread-list.visible")
        .await?
    {
        Some(conversation_list) => conversation_list,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Conversation list inmails not found".to_string(),
            ));
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    wait(3, 5);

    let mut conversations: HashMap<String, InmailConversation> = HashMap::new(); // hashmap to store conversations

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    scrap_conversation(
        conversation_list.inner_html().await?.as_str(),
        &api_key,
        &mut conversations,
    );

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////
    {
        if let Some(conversation) =
            conversations
                .iter()
                .find_map(|(_, conv)| if !conv.unread { Some(conv) } else { None })
        {
            if let Ok(Some(conv_element)) = conversation_list
                .query_selector(&format!("a[id='{}']", conversation.id))
                .await
            {
                conv_element.hover_builder();
                wait(1, 3);
                conv_element.click_builder().click().await?;
                wait(5, 9);
                scrap_stage(&browser, &api_key).await?;
            }
        };
    }

    wait(10, 12);

    for conversation in conversations.values() {
        wait(3, 7);

        match conversation_list
            .query_selector(format!("a[id='{}']", conversation.id).as_str())
            .await?
        {
            Some(conversation) => {
                conversation.hover_builder();
                wait(1, 3);
                conversation.click_builder().click().await?;
                wait(5, 12);
            }
            None => {
                return Err(CustomError::ButtonNotFound(
                    "Conversation not found".to_string(),
                ))
            }
        }; // select the conversation
           //
        if conversation.unread {
            match conversation_list
                .query_selector(format!("a[id='{}']", conversation.id).as_str())
                .await?
            {
                Some(conversation) => {
                    conversation.hover_builder();
                    wait(1, 3);
                    conversation.click_builder().click().await?;
                    wait(5, 12);

                    let conversation_id = "div[class='_card-container_z8knzq _active_z8knzq']";
                    let conversation_block =
                        browser.page.query_selector(conversation_id).await?.unwrap();
                    let unread_button = conversation_block.query_selector
            ("button[class='ember-view _button_ps32ck _small_ps32ck _tertiary_ps32ck _circle_ps32ck _container_iq15dg _flat_1aegh9 a11y-conversation-button']")
        .await?.unwrap();
                    unread_button.click_builder().click().await?;
                    // // ("button unread was pressed");
                }
                None => {
                    return Err(CustomError::ButtonNotFound(
                        "Conversation not found".to_string(),
                    ))
                }
            }; // select the conversation
        }
        let _fragment = true;
        //// ("unread {}", conversation.unread);
        // needs to be fixed for broken characters
        let messages_container = match browser
            .page
            .query_selector("div._messages-container_1j60am._divider_lvf5de")
            .await?
        {
            Some(conversation_list) => conversation_list,
            None => {
                wait(1, 5); // random delay
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Messaging container inmails not found".to_string(),
                ));
            } // if search input is not found, means page was not loaded and sessuion cookie is not valid
        };
        // ("conversation: {:?}", conversation);
        let html = messages_container.inner_html().await?;
        let full_name = FullName::split_name(conversation.candidate_name.as_str());
        let messages_text = scrap_message(conversation.clone(), html.as_str(), &full_name).unwrap();
        let text = messages_text.1;
        let messages = messages_text.0;
        for message in messages {
            create_message(&message).await?
        }
        if recruiter {
            let result = check_message(text.as_str(), &api_key, &full_name).await;
            match result {
                MessageCategory::Interested => {
                    // ("changing interested {:?}", result);
                    change_stage(&stage_interested, &browser).await?;
                }
                MessageCategory::NotInterested => {
                    // ("changing not-interested {:?}", result);
                    change_stage(&stage_not_interested, &browser).await?;
                }
                MessageCategory::NotFound => {
                    // ("No category found");
                }
            }
        }
    }
    wait(3, 7); // random delay

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}
struct InmailMessage {
    message_text: String,
    api_key: String,
    first_name: String,
    last_name: String,
    conversation_url: String,
}
fn scrap_message(
    conversation: InmailConversation,
    html: &str,
    name: &FullName,
) -> Result<(Vec<InmailMessage>, String), CustomError> {
    let document = Html::parse_document(html);
    let message_id_selector = Selector::parse("._message-list-item_1gj1uc").unwrap();
    let sender_name_selector = Selector::parse("._headingText_e3b563").unwrap();
    let timestamp_selector = Selector::parse("time").unwrap();
    let message_text_selector =
        Selector::parse("._message-data-wrapper_1gj1uc div div div").unwrap();

    let mut full_text = String::new();
    let mut messages: Vec<InmailMessage> = Vec::new();
    for message_element in document.select(&message_id_selector) {
        let mut sender_full_name = String::new();
        if let Some(sender_element) = message_element.select(&sender_name_selector).next() {
            sender_full_name = sender_element.inner_html();
        }

        if let Some(timestamp_element) = message_element.select(&timestamp_selector).next() {
            let _timestamp = timestamp_element.inner_html();
        }
        let mut message_text = String::new();
        if let Some(message_text_element) = message_element.select(&message_text_selector).next() {
            message_text = message_text_element.inner_html();
        }

        if conversation.candidate_name.trim() == sender_full_name.trim() {
            full_text.push_str(format!("Candidate: {} \n", message_text).as_str());
            let message = InmailMessage {
                message_text,
                api_key: conversation.api_key.clone(),
                first_name: name.first_name.clone(),
                last_name: name.last_name.clone(),
                conversation_url: conversation.thread_url.clone(),
            };
            messages.push(message);
        } else {
            full_text.push_str(format!("Recruiter: {} \n", message_text).as_str());
        }
    }
    // ("full text: {:?}", full_text);
    Ok((messages, full_text))
}

async fn check_message(text: &str, api: &str, name: &FullName) -> MessageCategory {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": text,
            "first": name.first_name,
            "last": name.last_name,
            "api_key": api

    });

    //// ("payload: {:?}", payload);
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/check_inmail")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the response

    let category = json_response["response"]["category"].as_str();
    // ("response: {:?}", json_response);
    match category {
        Some("Interested") => MessageCategory::Interested,
        Some("Not interested") => MessageCategory::NotInterested,
        Some(_) => MessageCategory::NotFound,
        None => MessageCategory::NotFound,
    }
}

async fn create_message(message: &InmailMessage) -> Result<(), CustomError> {
    let client = reqwest::Client::new();
    let payload = json!({
            "conversation_url": message.conversation_url,
            "message_text": message.message_text,
            "api_key": message.api_key,
            "first": message.first_name,
            "last": message.last_name,

    });

    //// ("payload: {:?}", payload);
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_receive_inmail")
        .json(&payload)
        .send()
        .await?;
    let _json_response: serde_json::Value = res.json().await?; //here is lays the response
    Ok(())
}
async fn change_stage(stage: &str, browser: &BrowserConfig) -> Result<(), CustomError> {
    wait(5, 6);
    // ("changing stage: {:?}", stage);
    let button_dropdown = browser.page.query_selector("div.artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-right.ember-view").await?;
    if button_dropdown.is_some() {
        button_dropdown.unwrap().click_builder().click().await?;
        wait(2, 3);

        let dropdown_list = browser
            .page
            .query_selector_all("div.artdeco-dropdown__item")
            .await?;

        for item in dropdown_list {
            let span_item = item
                .query_selector("span[data-live-test-change-stage-list-item='true']")
                .await?;
            match span_item {
                Some(span) => {
                    let text = span.inner_text().await?;
                    if text.trim() == stage.trim() {
                        // ("stage was found");
                        item.click_builder().click().await?;
                        // ("stage was clicked");
                        break;
                    }
                }
                None => (), // ("stage was not found"),
            }
        }
    }
    Ok(())
}

async fn scrap_stage(browser: &BrowserConfig, api_key: &str) -> Result<(), CustomError> {
    let button_dropdown = browser.page.query_selector("div.artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-right.ember-view").await?;
    if button_dropdown.is_some() {
        button_dropdown.unwrap().click_builder().click().await?;
        wait(2, 3);

        let dropdown_list = browser
            .page
            .query_selector_all("div.artdeco-dropdown__item")
            .await?;

        for item in dropdown_list {
            let span_item = item
                .query_selector("span[data-live-test-change-stage-list-item='true']")
                .await?;
            match span_item {
                Some(span) => {
                    let text = span.inner_text().await?;
                    let client = reqwest::Client::new();
                    let payload = json!({
                                    "api_key": api_key,
                                    "stage_name": text.trim(),

                    });
                    let _res = client
                        .post("https://overview.tribe.xyz/api/1.1/wf/create_inmail_stages")
                        .json(&payload)
                        .send()
                        .await
                        .unwrap();
                }
                None => (),
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum MessageCategory {
    Interested,
    NotInterested,
    NotFound,
}

fn scrap_conversation(
    html: &str,
    api_key: &str,
    conversations: &mut HashMap<String, InmailConversation>,
) {
    let document = Html::parse_document(html);

    let conversation_selector = Selector::parse("._card-container_z8knzq").unwrap();

    let name_selector = Selector::parse("._conversation-card-participant-name_z8knzq").unwrap();

    let url_selector = Selector::parse("._conversation-link_z8knzq").unwrap();
    let unread_selector = Selector::parse("._unread-badge_z8knzq").unwrap();
    let snippet_selector = Selector::parse("._conversation-snippet_z8knzq").unwrap();

    for conversation in document.select(&conversation_selector) {
        let id = conversation
            .select(&url_selector)
            .next()
            .map(|element| element.value().attr("id"))
            .unwrap_or(Some("Not found"));

        let name = conversation
            .select(&name_selector)
            .next()
            .map(|element| element.inner_html())
            .unwrap_or("Not found".to_string())
            .trim()
            .to_string();

        let url = conversation
            .select(&url_selector)
            .next()
            .map(|element| element.value().attr("href"))
            .unwrap_or(Some("Not found"));

        let unread = match conversation.select(&unread_selector).next() {
            Some(_) => true,
            None => false,
        };
        let _snippet = conversation
            .select(&snippet_selector)
            .next()
            .map(|element| element.inner_html())
            .unwrap_or("Not found".to_string());

        let conversation = InmailConversation::new(
            id.unwrap().to_string(),
            url.unwrap().to_string(),
            name,
            unread,
            api_key.to_string(),
        );

        conversations.insert(id.unwrap().to_string(), conversation);
    }
}
