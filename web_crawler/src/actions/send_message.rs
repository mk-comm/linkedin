use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};
use thirtyfour::{By, Key};

pub async fn send_message(entry: EntrySendConnection) -> Result<String, CustomError> {
    let check_reply = match entry.check_reply {
        Some(value) => value,
        None => false,
    };
    println!("{}", check_reply);
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

    let mut go_to = browser.goto(&candidate.linkedin).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(&candidate.linkedin).await;
            if build.is_ok() {
                go_to = build;
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                let screenshot = browser.screenshot_as_png().await?;
                browser.quit().await?;
                send_screenshot(
                    screenshot,
                    &browser_info.user_id,
                    "Candidate page is not loading/Send_regular_message",
                    "Send regular message",
                )
                .await?;

                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Send_regular_message".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            //// ("retrying to load page")
        }
        wait(1, 3);
    }
    wait(10, 15); // random delay
    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            wait(1, 3);
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Session cookie expired",
                "Send regular message",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const MAIN_CONTAINER: &str = "div[class=application-outlet]";
    let main_container = browser.find(By::Css(MAIN_CONTAINER)).await;
    if main_container.is_err() {
        let screenshot = browser.screenshot_as_png().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Main container not found/Send_regular_message",
            "Send regular message",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Main container not found/Send regular message".to_string(),
        ));
    }
    let entity_urn = find_entity_urn(&main_container.unwrap().inner_html().await?);
    if entity_urn.is_none() {
        browser.quit().await?;
        return Err(CustomError::ButtonNotFound(
            "Entity urn not found".to_string(),
        ));
    };
    const IN_CONNECTION_POOL: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Remove your connection']";
    let in_connection_pool = browser.find(By::Css(IN_CONNECTION_POOL)).await;
    if in_connection_pool.is_err() {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Candidate is not in connection pool to send messages",
            "Send regular message",
        )
        .await?;
        return Ok("Candidate is not in connection pool to send messages".to_string());
    }
    const MESSAGE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.pvs-profile-actions__action[aria-label*='Message']";
    let message_button = browser.find(By::Css(MESSAGE_BUTTON)).await;

    let message_button = match message_button {
        Ok(button) => button,
        Err(_e) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Message button not found",
                "Send regular message",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Message button not found".to_string(),
            ));
        } // means there is no message button
    };

    message_button.click().await?;
    wait(2, 5);
    const INMAIL_POPUP: &str = "a.app-aware-link.artdeco-button.artdeco-button--premium";
    let inmail_popup = browser.find(By::Css(INMAIL_POPUP)).await;

    if inmail_popup.is_ok() {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Inmail needed",
            "Send regular message",
        )
        .await?;
        return Err(CustomError::ButtonNotFound("Inmail needed".to_string()));
    }

    const PICK: &str = "aside.msg-overlay-container";
    let pick = browser.find(By::Css(PICK)).await?;
    let linkedin_url = browser
        .current_url()
        .await?
        .as_str()
        .replace("https://www.linkedin.com", "");
    let conversation_selector = format!(
        "div.relative.display-flex.flex-column.flex-grow-1:has(a[href='{}'])",
        linkedin_url
    );
    let mut candidate_reply = false;
    let html = pick.inner_html().await?;
    //let name = get_conversation_owner(html.as_str());
    let conversation_id = find_conversation(html.as_str(), entity_urn.unwrap().as_str());
    let conversation_select = match browser
        .find(By::Css(format!("div[id='{}']", conversation_id).as_str()))
        .await
    {
        Ok(conversation) => {
            let name = get_conversation_owner(html.as_str());
            candidate_reply = is_last_message_from_owner(
                conversation.inner_html().await?.as_str(),
                name.unwrap().as_str(),
            );
            Some(conversation)
        }
        Err(_s) => {
            let convesation_linkedin_nick = browser.find(By::Css(&conversation_selector)).await;
            match convesation_linkedin_nick {
                Ok(conversation) => Some(conversation),
                Err(_s) => {
                    let screenshot = browser.screenshot_as_png().await?;
                    browser.quit().await?;
                    send_screenshot(
                        screenshot,
                        &browser_info.user_id,
                        "Inmail needed",
                        "Send regular message",
                    )
                    .await?;
                    return Err(CustomError::ButtonNotFound(
                        "Conversation not found/ new or existing".to_string(),
                    ));
                }
            }
        }
    };
    if candidate_reply && check_reply {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Candidate replied",
            "Send regular message",
        )
        .await?;
        return Err(CustomError::ButtonNotFound("Candidate replied".to_string()));
    };
    let conversation_select = conversation_select.unwrap();
    wait(2, 4);
    const REGULAR_INPUT: &str = "div.msg-form__contenteditable.t-14.t-black--light.t-normal.flex-grow-1.full-height.notranslate";
    let regular_input = conversation_select.find(By::Css(REGULAR_INPUT)).await;

    match regular_input {
        Ok(input) => {
            input.focus().await?;
            wait(1, 2);
            input.click().await?;
            wait(1, 2);
            input.send_keys(Key::Control + "a").await?;
            wait(1, 2);
            input.send_keys(Key::Control + "x").await?;
            input.focus().await?;
            input.click().await?;
            wait(1, 3);
            //wait(10000, 200000); // random delay
            input.send_keys(&candidate.message).await?; // fill input for note;
        }
        Err(_s) => {
            wait(1, 5); // random delay
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Input not found",
                "Send regular message",
            )
            .await?;
            return Err(CustomError::ButtonNotFound("Input not found".to_string()));
        } // means you can't send message to this profile
    }
    const SEND: &str = "button.msg-form__send-button.artdeco-button.artdeco-button--1";
    let send = conversation_select.find(By::Css(SEND)).await;

    const SEND_NEW: &str = "button[class='msg-form__send-btn artdeco-button artdeco-button--circle artdeco-button--1 artdeco-button--primary ember-view']";
    let send_new = conversation_select.find(By::Css(SEND_NEW)).await;

    match send {
        Ok(send) => {
            send.click().await?;
            wait(2, 5);
        }
        Err(_s) => {
            match send_new {
                Ok(send) => {
                    send.click().await?; // click on search input
                    wait(2, 5); // random delay
                }
                Err(_s) => {
                    let screenshot = browser.screenshot_as_png().await?;
                    browser.quit().await?;
                    send_screenshot(
                        screenshot,
                        &browser_info.user_id,
                        "Send button not found",
                        "Send regular message",
                    )
                    .await?;

                    return Err(CustomError::ButtonNotFound(
                        "Send button not found".to_string(),
                    ));
                }
            }
        } // means you can't send message to this profile
    }

    wait(5, 7);
    browser.quit().await?;
    Ok("Message was sent".to_string())
}
fn get_conversation_owner(html: &str) -> Option<String> {
    // Parse the HTML document
    let document = Html::parse_document(html);

    // Create a selector to find the owner of the conversation
    let owner_selector = Selector::parse("h2.msg-overlay-bubble-header__title span").unwrap();

    // Find the owner element
    if let Some(owner_element) = document.select(&owner_selector).next() {
        // Get the text content of the owner element
        let owner = owner_element.text().collect::<Vec<_>>().join(" ");
        println!("owner {}", owner);
        let owner = owner.to_string().trim().to_string();
        println!("owner {}", owner);
        return Some(owner);
    }

    None
}

fn is_last_message_from_owner(html: &str, owner: &str) -> bool {
    // Parse the HTML document
    let document = Html::parse_document(html);

    // Create a selector to find all message items
    let message_selector = Selector::parse("li.msg-s-message-list__event").unwrap();
    let name_selector = Selector::parse("span.msg-s-message-group__name").unwrap();

    // Get all message items
    let messages: Vec<_> = document.select(&message_selector).collect();

    // Check if there are any messages
    if let Some(last_message) = messages.last() {
        // Find the name span within the last message
        if let Some(name_element) = last_message.select(&name_selector).next() {
            // Get the text content of the name span
            let name = name_element.text().collect::<Vec<_>>().join(" ");
            // Check if the name is the owner
            return name.contains(owner);
        }
    }

    // If no messages or the last message is not from the owner, return false
    false
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

fn find_entity_urn(html: &str) -> Option<String> {
    let link_selector = Selector::parse("a").unwrap();
    let document = scraper::Html::parse_document(&html);
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
        entity_urn = match print_elements_with_datalet_in_id(document.html().as_str()) {
            Some(urn) => urn,
            None => return None,
        };
    }
    Some(entity_urn)
}

fn print_elements_with_datalet_in_id(html: &str) -> Option<String> {
    // Parse the document
    let document = Html::parse_document(html);

    // Create a Selector for elements with an 'id' attribute
    let selector = match Selector::parse("[id]") {
        Ok(selector) => selector,
        Err(_) => return None,
    };

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

    let entity_id_selector = match Selector::parse(&right_id) {
        Ok(selector) => selector,
        Err(_) => return None,
    };
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

    Some(entity_urn)
}
