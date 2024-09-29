use crate::actions::wait::wait;
use crate::structs::conversation::Conversation;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use crate::structs::message::Message;

use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;
use thirtyfour::{By, WebDriver, WebElement};

pub async fn scrap_message(
    conversation: &Conversation,
    focused_inbox: bool,
    browser: &WebDriver,
) -> Result<(), CustomError> {
    let conversation_select = match browser
        .find(By::Css(format!("li[id='{}']", conversation.id).as_str()))
        .await
    {
        Ok(conversation) => {
            wait(1, 7);
            conversation.click().await?;
            conversation
        }
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Conversation select not found".to_string(),
            ))
        }
    }; // select the conversation

    wait(7, 19);
    const MESSAGE_CONTAINER: &str =
        "ul[class='msg-s-message-list-content list-style-none full-width mbA']";
    let message_container = match browser.find(By::Css(MESSAGE_CONTAINER)).await {
        Ok(container) => container,
        Err(_) => return Ok(()),
    };
    // select the message container
    const OWNER_CONTAINER: &str =
        "div[class='msg-title-bar global-title-container shared-title-bar']";
    let owner_container = browser.find(By::Css(OWNER_CONTAINER)).await?;

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    let mut full_text = String::new(); // Conversation text to push to AI
    let tuple = scrap_each_message(
        owner_container.inner_html().await?.as_str(),
        message_container.inner_html().await?.as_str(),
        &mut full_text,
    );
    let messages = tuple.1; //hashmap for storing all messages
    let conversation_owner_link = tuple.0;

    let candidate_of_sequence = scrap_profile(
        browser,
        conversation_owner_link.as_str(),
        &conversation.api_key,
    )
    .await?; // check if candidate is in sequence
    let pages = browser.windows().await?;
    browser.switch_to_window(pages[0].clone()).await?;
    conversation_select.click().await?;

    // checks if the message is new or was scraped before

    let full_name = FullName::split_name(conversation.candidate_name.as_str());

    let mut new_message = false;

    /////////////////loop for create/check message

    for message in messages.values() {
        let check_message = check_message_new_message(message, &full_name, conversation).await;
        if check_message == true && message.received == true {
            new_message = true;
            create_message(message, conversation).await;
        }
    }
    // check if the message is new

    //////////////////////////////////loop for create/check message

    if new_message == true && candidate_of_sequence == Some(true) && conversation.enable_ai == true
    {
        let category = evaluate(full_text.as_str(), &conversation.api_key, full_name.clone()).await;
        match category {
            MessageCategory::Interested => {
                mark_star(&conversation_select, focused_inbox).await?;
                if conversation.unread == true {
                    conversation_select.click().await?;
                    wait(3, 5);
                    mark_unread(&conversation_select, focused_inbox).await?;
                    //// ("Marked as unread/Interested");
                }
            }
            MessageCategory::NotInterested => {
                //// ("Nothing happened/NotInterested");
            }
            MessageCategory::NotFound => {
                //// ("Category NotFound");
            }
        }
    }

    if conversation.enable_ai == false && conversation.unread == true {
        mark_unread(&conversation_select, focused_inbox).await?;
    }

    if conversation.enable_ai == true && conversation.unread == true && new_message == false {
        conversation_select.click().await?;
        mark_unread(&conversation_select, focused_inbox).await?;
    }

    Ok(())
}

async fn create_message(message: &Message, conversation: &Conversation) {
    // make an api call to bubble
    let full_name = FullName::split_name(message.sender.as_str());
    let _client = reqwest::Client::new();
    let _payload = json!({
            "message_text": message.message_text,
            "candidate_entity_urn": message.url_send_from,
            "received": message.received,
            "sender": message.sender,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
    });
    // ("payload :{}", _payload);
    const CREATE_MESSAGE_URL: &str = "https://overview.tribe.xyz/api/1.1/wf/tribe_api_receive";
    //const CREATE_MESSAGE_URL: &str = "https://webhook.site/c58568dc-6357-4aa4-96c2-79d6f22c1ede";

    let _res = _client
        .post(CREATE_MESSAGE_URL)
        .json(&_payload)
        .send()
        .await
        .unwrap();
}

async fn check_message_new_message(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
) -> bool {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message.message_text,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "entity_urn": message.url_send_from,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_message")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the response

    json_response["response"]["new_message"].as_bool().unwrap()
}
#[allow(dead_code)]
async fn check_autoreply(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
) -> bool {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message.message_text,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "entity_urn": message.url_send_from,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_autoreply")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the

    json_response["response"]["autoreply"].as_bool().unwrap()
}
#[allow(dead_code)]

async fn get_prompt() -> String {
    let client = reqwest::Client::new();
    let payload = json!({});
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_prompt")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the

    json_response["response"]["prompt"].to_string()
}

async fn mark_unread(
    conversation_element: &WebElement,
    focused_inbox: bool,
) -> Result<(), CustomError> {
    const DROPDOWN: &str = "div[class='msg-conversation-card__inbox-shortcuts']";
    let dropdown = conversation_element.find(By::Css(DROPDOWN)).await;

    // click the dropdown
    match dropdown {
        Ok(dropdown) => {
            match dropdown.click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        Err(_) => (),
    }

    const INNER_CONTAINER: &str = "div[class=artdeco-dropdown__content-inner]";

    let inner_container = conversation_element.find(By::Css(DROPDOWN)).await;
    //find container for the buttons inside dropdown

    let inner_container = match inner_container {
        Ok(inner_container) => inner_container,
        Err(_) => return Ok(()),
    };

    //find mark unread button;
    const UNREAD_FOCUSED:&str = "div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(2)";
    const UNREAD_NOT_FOCUSED:&str = "div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(1)";

    let mark_unread_button = if focused_inbox == true {
        inner_container.find(By::Css(UNREAD_FOCUSED)).await
    } else {
        inner_container.find(By::Css(UNREAD_NOT_FOCUSED)).await
    };
    //click mark unread button
    match mark_unread_button {
        Ok(button) => {
            wait(1, 3);
            button.click().await?;
            Ok(())
        }
        Err(_) => Err(CustomError::ButtonNotFound(
            "Unread button in dropdown not found".to_string(),
        )),
    }
}

async fn mark_star(
    conversation_element: &WebElement,
    focused_inbox: bool,
) -> Result<(), CustomError> {
    const DROPDOWN: &str = "div[class='msg-conversation-card__inbox-shortcuts']";
    let dropdown = conversation_element.find(By::Css(DROPDOWN)).await;

    // click the dropdown
    match dropdown {
        Ok(dropdown) => {
            match dropdown.click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        Err(_) => (),
    }

    const INNER_CONTAINER: &str = "div[class=artdeco-dropdown__content-inner]";

    let inner_container = conversation_element.find(By::Css(DROPDOWN)).await;
    //find container for the buttons inside dropdown

    let inner_container = match inner_container {
        Ok(inner_container) => inner_container,
        Err(_) => return Ok(()),
    };

    //find mark unread button;
    const MARK_STAR_FOCUSED:&str = "div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(3)";
    const MARK_STAR_NOT_FOCUSED:&str = "div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(2)";

    let mark_star_button = if focused_inbox == true {
        inner_container.find(By::Css(MARK_STAR_FOCUSED)).await
    } else {
        inner_container.find(By::Css(MARK_STAR_NOT_FOCUSED)).await
    };

    //click mark unread button
    match mark_star_button {
        Ok(button) => {
            wait(1, 3);
            button.click().await?;
            Ok(())
        }
        Err(_) => Err(CustomError::ButtonNotFound(
            "Unread button in dropdown not found".to_string(),
        )),
    }
}
/*
async fn move_other(conversation_element: &ElementHandle) -> Result<(), CustomError> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => (),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => (),
    }

    //find container for the buttons inside dropdown
    let inner_container = conversation_element
        .query_selector("div[class=artdeco-dropdown__content-inner]")
        .await?;
    if inner_container.is_none() {
        return Ok(());
    }

    //find move to other button
    let move_other_button = inner_container.unwrap().query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(1)").await?;

    //click move to other button
    match move_other_button {
        Some(button) => {
            wait(1, 3);
            button.click_builder().click().await?;
            Ok(())
        }
        None => Err(CustomError::ButtonNotFound(
            "Move to other in dropdown not found".to_string(),
        )),
    }
}
*/
async fn scrap_profile(
    browser: &WebDriver,
    entity_urn: &str,
    api_key: &str,
) -> Result<Option<bool>, CustomError> {
    let page = browser.new_tab().await?;
    browser.switch_to_window(page).await?;
    let go_to = browser.goto(&entity_urn).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(&entity_urn).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Scrap messages".to_string(),
                ));
            }
            x += 1;
        }
        wait(1, 3);
    }

    wait(5, 12);
    const CONTACT_INFO: &str = "a#top-card-text-details-contact-info";
    let contact_info = browser.find(By::Css(CONTACT_INFO)).await?;

    let url = contact_info.attr("href").await?;

    let client = reqwest::Client::new();
    let payload = json!({
            "entity_urn": entity_urn,
            "linkedin": url,
    });
    const ENTITY_URN_URL: &str = "https://overview.tribe.xyz/api/1.1/wf/update_entity_urn";

    let _res = client
        .post(ENTITY_URN_URL)
        .json(&payload)
        .send()
        .await
        .unwrap();
    // check if candidate is a part of the sequence
    let client = reqwest::Client::new();
    let payload = json!({
            "api_key": api_key,
            "linkedin": url,
    });

    const CHECK_SEQUENCE_URL: &str =
        "https://overview.tribe.xyz/api/1.1/wf/tribe_api_check_sequence";
    let res = client
        .post(CHECK_SEQUENCE_URL)
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the

    let candidate_part_of_sequence = json_response["response"]["part_of_sequence"]
        .as_bool()
        .unwrap();
    browser.close_window().await?;
    Ok(Some(candidate_part_of_sequence))
}

#[derive(Debug)]
enum MessageCategory {
    Interested,
    NotInterested,
    NotFound,
}

async fn evaluate(full_text: &str, api: &str, name: FullName) -> MessageCategory {
    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": full_text,
            "first": name.first_name,
            "last": name.last_name,
            "api_key": api
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/check_inmail")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the
    let category = json_response["response"]["category"].as_str();
    match category {
        Some("Interested") => MessageCategory::Interested,
        Some("Not interested") => MessageCategory::NotInterested,
        Some(_) => MessageCategory::NotFound,
        None => MessageCategory::NotFound,
    }
}

fn scrap_each_message(
    html: &str,
    message_container: &str,
    full_text: &mut String,
) -> (String, HashMap<String, Message>) {
    let owner_container_html = html;
    let owner_document = Html::parse_document(owner_container_html);
    let owner_selector = Selector::parse("a.app-aware-link.msg-thread__link-to-profile").unwrap();

    let conversation_owner_link: String;

    if let Some(conversation_owner) = owner_document.select(&owner_selector).next() {
        conversation_owner_link = conversation_owner
            .value()
            .attr("href")
            .and_then(|href| Some(href.to_owned()))
            .unwrap_or_else(|| String::new());
    } else {
        conversation_owner_link = String::new();
    }

    let mut messages: HashMap<String, Message> = HashMap::new(); // list of messages
                                                                 //let mut full_text = String::new(); // Conversation text to push to AI
                                                                 //let mut new_message = false; // if true and candidate_of_sequence is true evaluate conversation

    // Selectors for the message container
    let message_container_html = message_container;
    let document = Html::parse_document(&message_container_html); // parse html
    let sender_selector =
        Selector::parse(".msg-s-message-group__meta .msg-s-message-group__profile-link").unwrap();
    let timestamp_selector = Selector::parse(".msg-s-message-group__meta time").unwrap();
    let content_selector = Selector::parse(".msg-s-event__content p").unwrap();
    let url_send_from_selector =
        Selector::parse(".msg-s-event-listitem__link[tabindex=\"0\"]").unwrap();
    let url_send_to_selector = Selector::parse(".msg-s-message-group__meta a").unwrap();
    //

    // select the conversation
    // Iterate over the message container and create a message
    for ((((sender, timestamp), content), url_send_from), url_send_to) in document
        .select(&sender_selector)
        .zip(document.select(&timestamp_selector))
        .zip(document.select(&content_selector))
        .zip(document.select(&url_send_from_selector))
        .zip(document.select(&url_send_to_selector))
    {
        let sender = sender.text().collect::<String>().trim().to_owned();
        //let full_name = FullName::split_name(&sender);
        let timestamp = timestamp.text().collect::<String>().trim().to_owned();
        let message_text = content.text().collect::<String>().trim().to_owned();
        let url_send_from = url_send_from.value().attr("href").unwrap().to_owned();
        let url_send_to = url_send_to.value().attr("href").unwrap().to_owned();
        // ("url_send_from: {}", url_send_from);
        // ("conversation_owner_link: {}", conversation_owner_link);
        let received: bool = if conversation_owner_link == url_send_from {
            true
        } else {
            false
        };
        // ("received became: {}", received);
        let message = Message {
            sender,
            timestamp,
            message_text,
            url_send_from,
            url_send_to,
            received,
        };
        // ("received in message: {}", message.received);

        if received == true {
            full_text.push_str(format!("Candidate: {}\n", &message.message_text.clone()).as_str())
        } else {
            full_text.push_str(format!("Recruiter: {}\n", &message.message_text.clone()).as_str())
        }

        messages.insert(format!("message_{}", messages.len() + 1), message);
    } // scrap message container end

    (conversation_owner_link, messages)
}
