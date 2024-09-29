use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryRecruiter;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use crate::structs::inmail_conversation::InmailConversation;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::scrap_recruiter_search::check_recruiter_cookie;
use thirtyfour::{By, WebDriver};

pub async fn scrap_inmails(entry: EntryRecruiter) -> Result<String, CustomError> {
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
    let browser = init_browser(&browser_info).await?;
    //wait(10000, 10000);
    let result = scrap(
        &browser,
        api_key.as_str(),
        recruiter,
        stage_interested,
        stage_not_interested,
    )
    .await;
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
async fn scrap(
    browser: &WebDriver,
    api_key: &str,
    recruiter: bool,
    stage_interested: String,
    stage_not_interested: String,
) -> Result<String, CustomError> {
    const INMAIL_URL:&str = "https://www.linkedin.com/talent/inbox/0/main/id/2-MTBiZjJhZTMtNTNlNi00NDRjLddWJmZGQtYTg5MTk4ZjA5MWExXzAxMg==";
    let go_to = browser.goto(INMAIL_URL).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(INMAIL_URL).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Inmail_regular".to_string(),
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

    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::RecruiterSessionCookieExpired);
        }
    }
    const CONVERSATION_LIST: &str = "div.thread-list.visible";
    let conversation_list = browser.find(By::Css(CONVERSATION_LIST)).await;

    let conversation_list = match conversation_list {
        Ok(conversation_list) => conversation_list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Conversation list inmails not found".to_string(),
            ));
        }
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
            if let Ok(conv_element) = conversation_list
                .find(By::Css(&format!("a[id='{}']", conversation.id)))
                .await
            {
                conv_element.click().await?;
                wait(5, 9);
                scrap_stage(&browser, &api_key).await?;
            }
        };
    }

    wait(10, 12);

    for conversation in conversations.values() {
        wait(3, 7);

        match conversation_list
            .find(By::Css(&format!("a[id='{}']", conversation.id)))
            .await
        {
            Ok(conversation) => {
                conversation.click().await?;
                wait(5, 12);
            }
            Err(_) => {
                return Err(CustomError::ButtonNotFound(
                    "Conversation not found".to_string(),
                ))
            }
        }; // select the conversation
           //
        if conversation.unread {
            match conversation_list
                .find(By::Css(&format!("a[id='{}']", conversation.id)))
                .await
            {
                Ok(conversation) => {
                    conversation.click().await?;
                    wait(5, 12);

                    const CONVERSATION_ID: &str =
                        "div[class='_card-container_z8knzq _active_z8knzq']";
                    let conversation_block = browser
                        .find(By::Css(&format!("a[id='{}']", CONVERSATION_ID)))
                        .await?;
                    const UNREAD_BUTTON:&str ="button[class='ember-view _button_ps32ck _small_ps32ck _tertiary_ps32ck _circle_ps32ck _container_iq15dg _flat_1aegh9 a11y-conversation-button']";
                    let unread_button = conversation_block
                        .find(By::Css(&format!("a[id='{}']", CONVERSATION_ID)))
                        .await?;
                    unread_button.click().await?;
                }
                Err(_) => {
                    return Err(CustomError::ButtonNotFound(
                        "Conversation not found".to_string(),
                    ))
                }
            }; // select the conversation
        }
        let _fragment = true;
        //// ("unread {}", conversation.unread);
        // needs to be fixed for broken characters
        const MESSAGE_CONTAINER: &str = "div._messages-container_1j60am._divider_lvf5de";
        let messages_container = match browser.find(By::Css(MESSAGE_CONTAINER)).await {
            Ok(conversation_list) => conversation_list,
            Err(_) => {
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

    Ok("Inmail Messages were scraped".to_string())
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
    //let message_text_selector =
    //Selector::parse("._message-data-wrapper_1gj1uc div div div").unwrap();
    let message_text_selector =
        Selector::parse("article[class=messaging-attributed-text-renderer]").unwrap();
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
        //.post("https://webhook.site/56d11347-8dd9-4a06-b365-4791b9faf62b")
        .json(&payload)
        .send()
        .await?;
    //let _json_response: serde_json::Value = res.json().await?; //here is lays the response
    Ok(())
}
async fn change_stage(stage: &str, browser: &WebDriver) -> Result<(), CustomError> {
    wait(5, 6);
    // ("changing stage: {:?}", stage);
    const BUTTON_DROPDOWN: &str = "div.artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-right.ember-view";
    let button_dropdown = browser.find(By::Css(BUTTON_DROPDOWN)).await;
    if button_dropdown.is_ok() {
        button_dropdown.unwrap().click().await?;
        wait(2, 3);
        const DROPDOWN_LIST: &str = "div.artdeco-dropdown__item";

        let dropdown_list = browser.find(By::Css(DROPDOWN_LIST)).await;

        for item in dropdown_list {
            const SPAN_ITEMS: &str = "span[data-live-test-change-stage-list-item='true']";
            let span_item = item.find(By::Css(SPAN_ITEMS)).await;

            match span_item {
                Ok(span) => {
                    let text = span.text().await?;
                    if text.trim() == stage.trim() {
                        // ("stage was found");
                        item.click().await?;
                        // ("stage was clicked");
                        break;
                    }
                }
                Err(_) => (), // ("stage was not found"),
            }
        }
    }
    Ok(())
}

async fn scrap_stage(browser: &WebDriver, api_key: &str) -> Result<(), CustomError> {
    const BUTTON_DROPDOWN: &str = "div.artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-right.ember-view";

    let button_dropdown = browser.find(By::Css(BUTTON_DROPDOWN)).await;
    if button_dropdown.is_ok() {
        button_dropdown.unwrap().click().await?;
        wait(2, 3);

        const DROPDOWN_LIST: &str = "div.artdeco-dropdown__item";

        let dropdown_list = browser.find(By::Css(DROPDOWN_LIST)).await;

        for item in dropdown_list {
            const SPAN_ITEMS: &str = "span[data-live-test-change-stage-list-item='true']";
            let span_item = item.find(By::Css(SPAN_ITEMS)).await;
            match span_item {
                Ok(span) => {
                    let text = span.text().await?;
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
                Err(_) => (),
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
