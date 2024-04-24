use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use playwright::api::Page;
use scraper::{Html, Selector};

use super::start_browser::start_browser;

pub async fn send_message(entry: EntrySendConnection) -> Result<(), CustomError> {
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );

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

    let search_input = browser
        .page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;
    match search_input {
        Some(search_input) => {
            search_input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            search_input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            search_input
                .fill_builder(&candidate.fullname)
                .fill()
                .await?; // fill search input with text
            wait(1, 5); // random delay
            search_input.press_builder("Enter").press().await?; // press Enter
            wait(2, 6); // random delay
        }
        None => {
            wait(1, 5); // random delay
        }
    };

    // go to candidate page
    browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await?;
    wait(3, 7); // random delay

    browser
        .page
        .wait_for_selector_builder("div.pv-top-card-v2-ctas"); // wait until the block with buttons is loaded

    let message_button = browser
        .page
        .query_selector("button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.pvs-profile-actions__action")
        //.query_selector("li-icon[type=send-privately]")
        .await;

    let entity_urn = match find_entity_run(&browser.page).await {
        Ok(entity_urn) => entity_urn,
        Err(_) => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Entity urn not found".to_string(),
            ));
        }
    };

    let message_button = match message_button {
        Ok(button) => match button {
            Some(button) => button,
            None => {
                wait(1, 5); // random delay
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?;
                return Err(CustomError::ButtonNotFound(
                    "Message button not found".to_string(),
                ));
            } // means there is no message button
        },
        Err(_) => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "message button(err) not found".to_string(),
            ));
        }
    };

    message_button.hover_builder(); // hover on search input
    wait(1, 4); // random delay
    message_button.click_builder().click().await?; // click on search input
    wait(2, 5); // random delay
                // Picking the right conversation
    let inmail_popup = browser
        .page
        .query_selector("a.app-aware-link.artdeco-button.artdeco-button--premium")
        .await?;

    if inmail_popup.is_some() {
        wait(1, 5); // random delay
        browser.page.close(Some(false)).await?;
        browser.browser.close().await?;
        //// ("You have to be premium to send messages to this profile");
        return Err(CustomError::ButtonNotFound("Inmail needed".to_string()));
    } // Inmail needed to send message to this profile
      // Get the HTML content of the messaging container
    let pick = browser
        .page
        .query_selector("aside.msg-overlay-container")
        .await?
        .unwrap();
    let html = pick.inner_html().await?;
    let conversation_id = find_conversation(html.as_str(), entity_urn.as_str());
    //// ("conversation_id: {}", conversation_id);

    let conversation_select = match browser
        .page
        .query_selector(format!("div[id='{}']", conversation_id).as_str())
        .await?
    {
        Some(conversation) => Some(conversation),
        None => {
            wait(4, 9); // random delay
            let linkedin_nick_div = browser
                .page
                .query_selector(".pv-text-details__about-this-profile-entrypoint")
                .await?
                .expect("Element with linkedin nick not found");
            let href_value = linkedin_nick_div
                .get_attribute("href")
                .await?
                .expect("Attribute not found");

            // Strip the URL to show only the LinkedIn nickname.
            // Assuming the href format is "/in/{nickname}/overlay/about-this-profile/"
            let linkedin_nick = href_value.split("/").nth(2).unwrap_or("");
            let conversation_id = find_conversation(html.as_str(), linkedin_nick);
            let conversation = browser
                .page
                .query_selector(format!("div[id='{}']", conversation_id).as_str())
                .await?;
            match conversation {
                Some(div) => Some(div),
                None => None,
            }
        }
    };
    let conversation_select = match conversation_select {
        Some(div) => div,
        None => {
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Conversation not found/ new or existing".to_string(),
            ));
        }
    };
    wait(2, 4);
    let regular_input = conversation_select
    .query_selector("div.msg-form__contenteditable.t-14.t-black--light.t-normal.flex-grow-1.full-height.notranslate")
    .await?;

    match regular_input {
        Some(input) => {
            input.hover_builder(); // hover on input for note
            wait(1, 3); // random delay
            input.focus().await?; // focus on input for note
            wait(1, 2); // random delay
            input.fill_builder(&candidate.message).fill().await?; // fill input for note;
            wait(1, 3);
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound("Input not found".to_string()));
        } // means you can't send message to this profile
    }

    let send = conversation_select
        .query_selector("button.msg-form__send-button.artdeco-button.artdeco-button--1")
        .await?;

    match send {
        Some(send) => {
            send.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            send.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Send button not found".to_string(),
            ));
        } // means you can't send message to this profile
    }

    wait(5, 7);
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}

fn find_conversation(html: &str, entity_urn: &str) -> String {
    // Parse the HTML content and find the required div
    let document = Html::parse_document(html);
    let conv_selector = Selector::parse("div.msg-convo-wrapper").unwrap();
    let href_selector = Selector::parse("a[href^='/in/']").unwrap();

    let code = format!("/in/{}/", entity_urn); //target URN
    let mut correct_div = String::new();
    for conv_div in document.select(&conv_selector) {
        let id = conv_div.value().attr("id").unwrap();
        //// ("{}", id);
        let href_elem = conv_div.select(&href_selector).next().unwrap();

        let href = href_elem.value().attr("href").unwrap();

        if href == code {
            //// (", {}", conv_div.inner_html());
            correct_div = id.to_owned();
            //let button = container.query_selector("button[class='msg-form__send-toggle artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--1 artdeco-button--tertiary ember-view']").await?.unwrap();
            //   button.click_builder();
        }
    }

    correct_div
}

async fn find_entity_run(page: &Page) -> Result<String, playwright::Error> {
    let link_selector = Selector::parse("a").unwrap();
    let document = scraper::Html::parse_document(&page.content().await?);
    let mut entity_urn = String::new();

    for link in document.select(&link_selector) {
        let href = link.value().attr("href").unwrap_or_default();
        if href.contains("profileUrn=") {
            let parts: Vec<&str> = href
                .split("?profileUrn=urn%3Ali%3Afsd_profile%3A")
                .collect();
            if parts.len() > 1 {
                entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                if entity_urn.is_empty() {
                    let parts: Vec<&str> = href
                        .split("?profileUrn=urn%3Ali%3Afs_normalized_profile%3A")
                        .collect();
                    if parts.len() > 1 {
                        entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                    }
                }
            }
            if !entity_urn.is_empty() {
                break;
            }
        }
    }

    if entity_urn.is_empty() {
        entity_urn = print_elements_with_datalet_in_id(document.html().as_str());
    }
    Ok(entity_urn)
}

fn print_elements_with_datalet_in_id(html: &str) -> String {
    // Parse the document
    let document = Html::parse_document(html);

    // Create a Selector for elements with an 'id' attribute
    let selector = Selector::parse("[id]").unwrap();

    let mut right_id = String::new();
    // Iterate over elements matching the selector
    for element in document.select(&selector) {
        if let Some(id_attr) = element.value().attr("id") {
            if id_attr.contains("datalet")
                && element
                    .html()
                    .contains("/voyager/api/identity/dash/profile")
            {
                let element_html: String = element.html();
                match element_html.find("bpr-guid-") {
                    Some(start) => match element_html[start..].find("\"") {
                        Some(end) => {
                            let end = end + start;
                            right_id = format!("[id={}]", &element_html[start..end]);
                        }
                        None => (), // ("Could not find end quote"),
                    },
                    None => (), // ("Could not find 'bpr-guid-'"),
                }
            }
        }
    }

    let entity_id_selector = Selector::parse(&right_id).unwrap();
    let mut entity_urn = String::new();
    for element in document.select(&entity_id_selector) {
        let text = element.html();
        let text_str = text.as_str();

        if let Some(start) = text_str.find("\"*elements\":[\"urn:li:fsd_profile:") {
            let start = start + "\"*elements\":[\"urn:li:fsd_profile:".len();
            if let Some(end) = text_str[start..].find("\"") {
                let end = start + end;
                entity_urn = text_str[start..end].to_string();
            }
        }
    }

    entity_urn
}
