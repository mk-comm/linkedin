use crate::actions::wait::wait;
use crate::structs::conversation::Conversation;
use crate::structs::fullname::FullName;
use crate::structs::message::Message;
use crate::structs::browser::BrowserConfig;
use playwright::api::ElementHandle;
use playwright::api::Page;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;

pub async fn scrap_message(
    conversation: &Conversation,
    page: &Page,
    focused_inbox: bool,
    browser: &BrowserConfig,
) -> Result<(), playwright::Error> {
        
    let conversation_select = match page
        .query_selector(format!("li[id='{}']", conversation.id).as_str())
        .await?
    {
        Some(conversation) => {
            conversation.hover_builder();
            wait(1, 3);
            conversation.click_builder().click().await?;
            conversation
        }
        None => return Err(playwright::Error::ObjectNotFound),
    }; // select the conversation

    wait(3, 7);

    let linkedin_nick = 

    let message_container = page
        .query_selector("ul[class='msg-s-message-list-content list-style-none full-width mbA']")
        .await
        .unwrap()
        .unwrap(); // select the message container

    let owner_container = page
        .clone()
        .query_selector("div[class='msg-title-bar global-title-container shared-title-bar']")
        .await
        .unwrap()
        .unwrap();

    let owner_container_html = owner_container.inner_html().await.unwrap();
    let owner_document = Html::parse_document(&owner_container_html);
    let owner_selector = Selector::parse("a.app-aware-link.msg-thread__link-to-profile").unwrap();

    let conversation_owner_link: String;

    if let Some(conversation_owner) = owner_document.select(&owner_selector).next() {
        conversation_owner_link = conversation_owner
            .value()
            .attr("href")
            .and_then(|href| Some(href.to_owned()))
            .unwrap_or_else(|| String::new());
        // Do something with conversation_owner_link
    } else {
        // Handle the case where there is no conversation owner element
        conversation_owner_link = String::new();
    }

    println!("conversation_owner_link: {}", &conversation_owner_link);

    check_urn(&conversation_owner_link, browser).await;

    let message_container_html = message_container.inner_html().await.unwrap();

    let document = Html::parse_document(&message_container_html); // parse html

    let sender_selector =
        Selector::parse(".msg-s-message-group__meta .msg-s-message-group__profile-link").unwrap();
    let timestamp_selector = Selector::parse(".msg-s-message-group__meta time").unwrap();
    let content_selector = Selector::parse(".msg-s-event__content p").unwrap();
    let url_send_from_selector =
        Selector::parse(".msg-s-event-listitem__link[tabindex=\"0\"]").unwrap();
    let url_send_to_selector = Selector::parse(".msg-s-message-group__meta a").unwrap();

    let mut messages: HashMap<String, Message> = HashMap::new();

    let mut full_text = String::new();
    
    let mut new_message = false; // if true evaluate conversation with AI

    let mut part_of_sequence = false; //

    for ((((sender, timestamp), content), url_send_from), url_send_to) in document
        .select(&sender_selector)
        .zip(document.select(&timestamp_selector))
        .zip(document.select(&content_selector))
        .zip(document.select(&url_send_from_selector))
        .zip(document.select(&url_send_to_selector))
    {
        let sender = sender.text().collect::<String>().trim().to_owned();

        let full_name = FullName::split_name(&sender);

        let timestamp = timestamp.text().collect::<String>().trim().to_owned();

        let message_text = content.text().collect::<String>().trim().to_owned();

        let url_send_from = url_send_from.value().attr("href").unwrap().to_owned();

        let url_send_to = url_send_to.value().attr("href").unwrap().to_owned();
        println!("conversation_owner: {}", conversation_owner_link);
        println!("send_from: {}", url_send_from);
        let received: bool = if conversation_owner_link == url_send_from {
            true
        } else {
            false
        };
        println!("received: {}", received);
        let message = Message {
            sender,
            timestamp,
            message_text,
            url_send_from,
            url_send_to,
            received,
        };

        if received == true {
            full_text.push_str(format!("Candidate: {}\n", &message.message_text.clone()).as_str())
        } else {
            full_text.push_str(format!("Recruiter: {}\n", &message.message_text.clone()).as_str())
        }
        
        // checks if the message is new or was scraped before
        let check_message: ResponseApi = check_message(&message, &full_name, conversation).await; // check if the message is new mark conversation as new
        create_message(&message, &full_name, conversation).await;
        println!("check_message: {:?}", check_message);

        /* 
        if check_message.new_message == true {
            if check_message.part_of_sequence == true {
                create_message(&message, &full_name, &conversation).await
            } else {
                //mark conversation unread if it was unread and return early
                /// Check if candidate a part of a sequence, yes evaluate conversation(mark read on unread) if not mark initial read/unread
                /// Check if message is new - create new


                //
                ()
            }
        } else {
            ()
        }
        */

        messages.insert(format!("message_{}", messages.len() + 1), message);
    }

    //println!("{}" , full_text);
//////////////////////////////////////////////////////////////////////
let client = reqwest::Client::new();
let payload = json!({
    "text": full_text,

});

let res = client
.post("https://overview.tribe.xyz/version-test/api/1.1/wf/zz_conv")
.json(&payload)
.send()
.await
.unwrap();
/// 
/// 
/// 
    println!("Messages: {:#?}", messages.len());


    println!("Scraping message done succesfuly");
    Ok(())
}

async fn create_message(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
)  {
    // make an api call to bubble
    // return interested or not interested

    let client = reqwest::Client::new();
    let payload = json!({
            "message_text": message.message_text,
            "candidate_entity_urn": message.url_send_from,
            "received": message.received,
            "sender": full_name.full_name,
            "first": full_name.first_name,
            "last": full_name.last_name,
            "conversation_url": conversation.thread_url,
            "api_key": conversation.api_key,

    });
    /*  
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_receive")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce
    *//*
    let category = json_response["response"]["category"].as_str();
    if conversation.enable_ai == false {
        return MessageCategory::NotFound;
    }
    match category {
        Some("Interested") => MessageCategory::Interested,
        Some("Not interested") => MessageCategory::NotInterested,
        Some(_) => MessageCategory::NotFound,
        None => MessageCategory::NotFound,
    }
    */
}

async fn check_message(
    message: &Message,
    full_name: &FullName,
    conversation: &Conversation,
) -> ResponseApi {
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
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    let new_message = json_response["response"]["new_message"].as_bool().unwrap();
    let part_of_sequence = json_response["response"]["part_of_sequence"]
        .as_bool()
        .unwrap();
    ResponseApi {
        new_message,
        part_of_sequence,
    }
}

async fn mark_unread(conversation_element: &ElementHandle, focused_inbox: bool) -> Result<(), playwright::Error> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => println!("dropdown clicked"),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => println!("Dropdown variable is not found: "),
    }

    //find container for the buttons inside dropdown
    let inner_container = conversation_element
        .query_selector("div[class=artdeco-dropdown__content-inner]")
        .await?;
    let inner_container = match inner_container{
        Some(inner_container) => inner_container,
        None => return Ok(()),
    };

    //find mark unread button;

    let mark_unread_button = if focused_inbox == true {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(5)").await?
    } else {
        inner_container.query_selector("div.msg-thread-actions__dropdown-option.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view:nth-child(4)").await?
    };

    //click mark unread button
    match mark_unread_button {
        Some(button) => {
            wait(1, 3);
            button.click_builder().click().await?;
            Ok(())
        }
        None => {
            println!("Unread button not found");
            Err(playwright::Error::ObjectNotFound)
        }
    }
}

async fn _move_other(conversation_element: &ElementHandle) -> Result<(), playwright::Error> {
    let dropdown = conversation_element
        .query_selector("div[class='msg-conversation-card__inbox-shortcuts']")
        .await?; // find 3 dots button

    // click the dropdown
    match dropdown {
        Some(dropdown) => {
            dropdown.hover_builder();
            wait(1, 3);
            match dropdown.click_builder().click().await {
                Ok(_) => println!("dropdown clicked"),
                Err(_) => return Ok(()),
            };
            wait(1, 3)
        }
        None => println!("Dropdown variable is not found: "),
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
        None => {
            println!("Move to other button not found");
            Err(playwright::Error::ObjectNotFound)
        }
    }
}

async fn check_urn(entity_urn: &str, browser: &BrowserConfig) {

    let client = reqwest::Client::new();
    let payload = json!({
            "entity_urn": entity_urn,
    });
    let res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/check_entity_urn")
        .json(&payload)
        .send()
        .await
        .unwrap();
    let json_response: serde_json::Value = res.json().await.unwrap(); //here is lays the responce

    let talent_exist = json_response["response"]["talent_exist"].as_bool().unwrap();

    if talent_exist == true {
        println!("Talent found");
    } else {
        scrap_profile(browser, entity_urn).await;
    }

}

async fn scrap_profile(browser: &BrowserConfig, entity_urn: &str) -> Result<String, playwright::Error> {

    let page = browser.context.new_page().await?;

    match page
    .goto_builder(&entity_urn)
    .goto()
    .await
    {
        Ok(_) => println!("Page loaded"),
        Err(_) => {
            page.close(Some(false)).await?;
            return Err(playwright::Error::ObjectNotFound)
        },
    };

    wait(5, 7);

   let contact_info = page.query_selector("a#top-card-text-details-contact-info").await?.unwrap();
   let url = contact_info.get_attribute("href").await?;

   let client = reqwest::Client::new();
    let payload = json!({
            "entity_urn": entity_urn,
            "linkedin": url,
    });
    let _res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/update_entity_urn")
        .json(&payload)
        .send()
        .await
        .unwrap();
    page.close(Some(false)).await?;
Ok(url.unwrap())
}



#[derive(Debug)]
struct ResponseApi {
    new_message: bool,
    part_of_sequence: bool,
}

enum MessageCategory {
    Interested,
    NotInterested,
    NotFound,
}

