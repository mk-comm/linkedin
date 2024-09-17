use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use percent_encoding::percent_decode_str;
use scraper::{Html, Selector};
use serde_json::json;
use thirtyfour::{By, Key, WebDriver, WebElement};
pub async fn send_message(entry: EntrySendConnection) -> Result<String, CustomError> {
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
    let result = message(&browser, &candidate, entry.check_reply).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Message was sent",
                &entry.message_id,
                "Send regular message",
            )
            .await?;

            return Ok(text);
        }
        Err(error) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                error.to_string().as_str(),
                &entry.message_id,
                "Send regular message",
            )
            .await?;
            return Err(error);
        }
    }
}

pub async fn message(
    browser: &WebDriver,
    candidate: &Candidate,
    check_reply: Option<bool>,
) -> Result<String, CustomError> {
    let check_reply = match check_reply {
        Some(value) => value,
        None => false,
    };
    let go_to = browser.goto(&candidate.linkedin).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(&candidate.linkedin).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Send_regular_message".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
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
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const MAIN_CONTAINER: &str = "div[class=application-outlet]";
    let main_container = browser.find(By::Css(MAIN_CONTAINER)).await;

    let main_container = match main_container {
        Ok(container) => container,
        Err(_s) => {
            return Err(CustomError::ButtonNotFound(
                "Main container not found/Send regular message".to_string(),
            ));
        }
    };

    const ENTITY_URN_CONTAINER:&str ="a.optional-action-target-wrapper.artdeco-button.artdeco-button--tertiary.artdeco-button--standard.artdeco-button--2.artdeco-button--muted.inline-flex.justify-center.full-width.align-items-center.artdeco-button--fluid";

    let entity_urn_container = browser.find(By::Css(ENTITY_URN_CONTAINER)).await;
    let entity_urn = match entity_urn_container {
        Ok(element) => match find_entity_urn_sidebar(&element).await {
            Ok(urn) => urn,
            Err(_s) => match find_entity_urn(&main_container.inner_html().await?) {
                Ok(urn) => urn,
                Err(_s) => {
                    return Err(CustomError::ButtonNotFound(
                        "Entity urn not found".to_string(),
                    ));
                }
            },
        },
        Err(_) => match find_entity_urn(&main_container.inner_html().await?) {
            Ok(urn) => urn,
            Err(_s) => {
                return Err(CustomError::ButtonNotFound(
                    "Entity urn not found".to_string(),
                ));
            }
        },
    };
    const IN_CONNECTION_POOL: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Remove your connection']";
    let in_connection_pool = browser.find(By::Css(IN_CONNECTION_POOL)).await;
    if in_connection_pool.is_err() {
        return Ok("Candidate is not in connection pool to send messages".to_string());
    }
    const MESSAGE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.pvs-profile-actions__action[aria-label*='Message']";
    let message_button = browser.find(By::Css(MESSAGE_BUTTON)).await;

    let message_button = match message_button {
        Ok(button) => button,
        Err(_e) => {
            return Err(CustomError::ButtonNotFound(
                "Message button not found".to_string(),
            ));
        } // means there is no message button
    };

    message_button.click().await?;
    wait(15, 20);
    let current_url = browser.current_url().await?;
    let result_url = current_url.as_str();
    if result_url.contains("messaging") {
        let result = send_messaging_page(&browser, check_reply, &candidate.message).await;
        if let Err(error) = result {
            return Err(error);
        }
    } else {
        let result = send_current_page(
            &browser,
            entity_urn.as_str(),
            check_reply,
            &candidate.message,
        )
        .await;
        if let Err(error) = result {
            return Err(error);
        }
    }

    wait(5, 7);
    Ok("Message was sent".to_string())
}
fn get_conversation_owner(html: &str, selector: &str) -> Result<String, CustomError> {
    // Parse the HTML document
    let document = Html::parse_document(html);

    // Create a selector to find the owner of the conversation
    let owner_selector = Selector::parse(selector)?;

    // Find the owner element
    if let Some(owner_element) = document.select(&owner_selector).next() {
        // Get the text content of the owner element
        let owner = owner_element.text().collect::<Vec<_>>().join(" ");
        println!("owner {}", owner);
        let owner = owner.to_string().trim().to_string();
        println!("owner {}", owner);
        return Ok(owner);
    }

    return Err(CustomError::ButtonNotFound(
        "get_conversation_owner".to_string(),
    ));
}

fn get_conversation_ids(html: &str) -> Result<Vec<String>, CustomError> {
    let document = Html::parse_document(html);
    const CONVERSATION_CLOSE_SELECTOR: &str = "button[class='msg-overlay-bubble-header__control artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--1 artdeco-button--tertiary ember-view']";
    let button_selector = Selector::parse(CONVERSATION_CLOSE_SELECTOR)?;
    let mut elements = vec![];
    for element in document.select(&button_selector) {
        match element.value().id() {
            Some(id) => elements.push(id.to_string()),
            None => (),
        };
    }
    Ok(elements)
}

async fn send_messaging_page(
    browser: &WebDriver,
    check_reply: bool,
    message: &str,
) -> Result<(), CustomError> {
    const INMAIL_POPUP: &str = "a.app-aware-link.artdeco-button.artdeco-button--premium";
    let inmail_popup = browser.find(By::Css(INMAIL_POPUP)).await;

    if inmail_popup.is_ok() {
        return Err(CustomError::ButtonNotFound("Inmail needed".to_string()));
    }
    const MAIN_CONTAINER: &str = "div[class=application-outlet]";
    let html = browser.find(By::Css(MAIN_CONTAINER)).await?;
    let html = html.inner_html().await?;
    let elements = get_conversation_ids(&html)?;
    if elements.len() > 0 {
        for element in elements {
            let button = browser.find(By::Id(&element)).await;
            match button {
                Ok(button) => button.click().await?,
                Err(_s) => (),
            }
            wait(1, 2);
        }
    };
    const CONVERSATION_SELECTOR: &str = "div.relative.display-flex.flex-column.flex-grow-1";
    let conversation = match browser.find(By::Css(CONVERSATION_SELECTOR)).await {
        Ok(conversation) => conversation,
        Err(_s) => {
            return Err(CustomError::ButtonNotFound(
                "Conversation not found/Messaging page".to_string(),
            ));
        }
    };
    let html = conversation.inner_html().await?;
    let current_url = browser.current_url().await?;
    let current_url = current_url.as_str();
    let new_thread = current_url.contains("messaging/thread/new/");
    let name = match new_thread {
        true => "".to_string(),
        false => get_conversation_owner(&html, "h2[id=thread-detail-jump-target]")?,
    };
    let candidate_reply = match new_thread {
        true => false,
        false => is_last_message_from_owner(&html, name.as_str())?,
    };
    println!("candidate reply {}", candidate_reply);
    if candidate_reply && check_reply {
        return Err(CustomError::ButtonNotFound("Candidate replied".to_string()));
    };

    wait(2, 4);
    const REGULAR_INPUT: &str = "div.msg-form__contenteditable.t-14.t-black--light.t-normal.flex-grow-1.full-height.notranslate";
    let regular_input = browser.find(By::Css(REGULAR_INPUT)).await;

    let regular_input = match regular_input {
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
            let script = r#"
        const input = arguments[0];
        const text = arguments[1];
        if (input) {
            input.focus(); // Focus on the input field
            input.click(); // Click on the input field to ensure it is active
            document.execCommand('insertText', false, text); // Insert text at the cursor position
        }
    "#;
            // The message to add to the input
            let regular_input_value = serde_json::to_value(input.clone()).unwrap();
            let message_value = json!(message);

            // Execute the JavaScript with the input element and the text to add
            browser
                .execute(script, vec![regular_input_value, message_value])
                .await?;
            // Execute the JavaScript with the input element and the text to add
            //input.send_keys(&message).await?;
            input
        }
        Err(_s) => {
            return Err(CustomError::ButtonNotFound("Input not found".to_string()));
        } // means you can't send message to this profile
    };
    wait(1, 2);
    const PRESS_ENTER_TO_SEND: &str = "div.msg-form__hint-text.t-12.t-black--light.t-normal";
    let press_enter_to_send = browser.find(By::Css(PRESS_ENTER_TO_SEND)).await;

    const SEND: &str = "button.msg-form__send-button.artdeco-button.artdeco-button--1";
    let send = browser.find(By::Css(SEND)).await;

    const SEND_NEW: &str = "button[class='msg-form__send-btn artdeco-button artdeco-button--circle artdeco-button--1 artdeco-button--primary ember-view']";
    let send_new = browser.find(By::Css(SEND_NEW)).await;

    match send {
        Ok(send) => {
            send.click().await?;
            wait(2, 5);
        }
        Err(_) => {
            match send_new {
                Ok(send) => {
                    send.click().await?; // click on search input
                    wait(2, 5); // random delay
                }
                Err(_s) => match press_enter_to_send {
                    Ok(_) => {
                        regular_input.send_keys(Key::Enter + "").await?;
                    }
                    Err(_) => {
                        return Err(CustomError::ButtonNotFound(
                            "Send button not found".to_string(),
                        ));
                    }
                },
            }
        }
    }
    Ok(())
}
async fn send_current_page(
    browser: &WebDriver,
    entity_urn: &str,
    check_reply: bool,
    message: &str,
) -> Result<(), CustomError> {
    const INMAIL_POPUP: &str = "a.app-aware-link.artdeco-button.artdeco-button--premium";
    let inmail_popup = browser.find(By::Css(INMAIL_POPUP)).await;

    if inmail_popup.is_ok() {
        return Err(CustomError::ButtonNotFound("Inmail needed".to_string()));
    }

    const PICK: &str = "aside.msg-overlay-container";
    let pick = browser.find(By::Css(PICK)).await?;
    let linkedin_url = browser
        .current_url()
        .await?
        .as_str()
        .replace("https://www.linkedin.com", "");
    let linkedin_url = percent_decode_str(linkedin_url.as_str()).decode_utf8_lossy();

    let conversation_selector = format!(
        "div.relative.display-flex.flex-column.flex-grow-1:has(a[href='{}'])",
        linkedin_url
    );
    let mut candidate_reply = false;
    let html = pick.inner_html().await?;
    //let name = get_conversation_owner(html.as_str());
    let conversation_id = find_conversation(html.as_str(), entity_urn);
    let conversation_id = match conversation_id {
        Ok(id) => id,
        Err(e) => {
            return Err(CustomError::ButtonNotFound(e.to_string()));
        }
    };
    println!("id: {}", conversation_id);
    let conversation_select = match browser
        .find(By::Css(format!("div[id='{}']", conversation_id).as_str()))
        .await
    {
        Ok(conversation) => {
            let name =
                get_conversation_owner(html.as_str(), "h2.msg-overlay-bubble-header__title span")?;
            candidate_reply = is_last_message_from_owner(
                conversation.inner_html().await?.as_str(),
                name.as_str(),
            )?;
            Some(conversation)
        }
        Err(_s) => {
            let convesation_linkedin_nick = browser.find(By::Css(&conversation_selector)).await;
            match convesation_linkedin_nick {
                Ok(conversation) => Some(conversation),
                Err(_s) => {
                    return Err(CustomError::ButtonNotFound(
                        "Conversation not found/ new or existing".to_string(),
                    ));
                }
            }
        }
    };
    if candidate_reply && check_reply {
        return Err(CustomError::ButtonNotFound("Candidate replied".to_string()));
    };
    let conversation_select = match conversation_select {
        Some(conversation) => conversation,
        None => {
            return Err(CustomError::ButtonNotFound(
                "conversation_select not found".to_string(),
            ));
        }
    };
    wait(2, 4);
    const REGULAR_INPUT: &str = "div.msg-form__contenteditable.t-14.t-black--light.t-normal.flex-grow-1.full-height.notranslate";
    let regular_input = conversation_select.find(By::Css(REGULAR_INPUT)).await;

    let regular_input = match regular_input {
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
            /*
                for line in message.split('\n') {
                    input.send_keys(line).await?;
                    input.send_keys("" + Key::PageDown).await?;
                    wait(1, 2);
                    input.send_keys("" + Key::Shift + Key::Enter).await?;
                }
            */
            let script = r#"
        const input = arguments[0];
        const text = arguments[1];
        if (input) {
            input.focus(); // Focus on the input field
            input.click(); // Click on the input field to ensure it is active
            document.execCommand('insertText', false, text); // Insert text at the cursor position
        }
    "#;
            // The message to add to the input
            let regular_input_value = serde_json::to_value(input.clone()).unwrap();
            let message_value = json!(message);

            // Execute the JavaScript with the input element and the text to add
            browser
                .execute(script, vec![regular_input_value, message_value])
                .await?;
            // Execute the JavaScript with the input element and the text to add
            //input.send_keys(&message).await?;
            input
        }
        Err(_s) => {
            return Err(CustomError::ButtonNotFound("Input not found".to_string()));
        }
    };
    wait(1, 2);
    const PRESS_ENTER_TO_SEND: &str = "div.msg-form__hint-text.t-12.t-black--light.t-normal";
    let press_enter_to_send = conversation_select.find(By::Css(PRESS_ENTER_TO_SEND)).await;

    const SEND: &str = "button.msg-form__send-button.artdeco-button.artdeco-button--1";
    let send = conversation_select.find(By::Css(SEND)).await;

    const SEND_NEW: &str = "button[class='msg-form__send-btn artdeco-button artdeco-button--circle artdeco-button--1 artdeco-button--primary ember-view']";
    let send_new = conversation_select.find(By::Css(SEND_NEW)).await;

    match send {
        Ok(send) => {
            send.click().await?;
            wait(2, 5);
        }
        Err(_) => {
            match send_new {
                Ok(send) => {
                    send.click().await?; // click on search input
                    wait(2, 5); // random delay
                }
                Err(_s) => match press_enter_to_send {
                    Ok(_) => {
                        regular_input.send_keys(Key::Enter + "").await?;
                    }
                    Err(_) => {
                        return Err(CustomError::ButtonNotFound(
                            "Send button not found".to_string(),
                        ));
                    }
                },
            }
        }
    }
    Ok(())
}

fn is_last_message_from_owner(html: &str, owner: &str) -> Result<bool, CustomError> {
    // Parse the HTML document
    let document = Html::parse_document(html);

    // Create a selector to find all message items
    let message_selector = Selector::parse("li.msg-s-message-list__event")?;
    let name_selector = Selector::parse("span.msg-s-message-group__name")?;

    // Get all message items
    let messages: Vec<_> = document.select(&message_selector).collect();

    // Check if there are any messages
    if let Some(last_message) = messages.last() {
        // Find the name span within the last message
        if let Some(name_element) = last_message.select(&name_selector).next() {
            // Get the text content of the name span
            let name = name_element.text().collect::<Vec<_>>().join(" ");
            // Check if the name is the owner
            return Ok(name.contains(owner));
        }
    }

    // If no messages or the last message is not from the owner, return false
    Ok(false)
}
fn find_conversation(html: &str, entity_urn: &str) -> Result<String, CustomError> {
    // Parse the HTML content and find the required div
    let document = Html::parse_document(html);
    let conv_selector = Selector::parse("div.msg-convo-wrapper")?;
    let href_selector = Selector::parse("a[href^='/in/']")?;

    let code = format!("/in/{}/", entity_urn); //target URN
    let mut correct_div = String::new();
    for conv_div in document.select(&conv_selector) {
        let id = conv_div.value().attr("id");
        let id = match id {
            Some(id) => id,
            None => {
                return Err(CustomError::ButtonNotFound(
                    "id = conv_div.value()".to_string(),
                ));
            }
        };
        let href_elem = conv_div.select(&href_selector).next();
        let href_elem = match href_elem {
            Some(elem) => elem,
            None => {
                return Err(CustomError::ButtonNotFound(
                    "href_elem = conv_div.select".to_string(),
                ));
            }
        };
        let href = href_elem.value().attr("href");
        let href = match href {
            Some(href) => href,
            None => {
                return Err(CustomError::ButtonNotFound(
                    "href = href_elem.value()".to_string(),
                ));
            }
        };

        if href == code {
            correct_div = id.to_owned();
        }
    }

    Ok(correct_div)
}

fn find_entity_urn(html: &str) -> Result<String, CustomError> {
    let link_selector = Selector::parse("a")?;
    let document = scraper::Html::parse_document(&html);
    let mut entity_urn = String::new();

    for link in document.select(&link_selector) {
        let href = link.value().attr("href");
        let href = match href {
            Some(href) => href,
            None => {
                return Err(CustomError::ButtonNotFound(
                    "let href = link.value().attr(href)".to_string(),
                ))
            }
        };
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
            None => {
                return Err(CustomError::ButtonNotFound(
                    "entity_urn = match print_elements_with_datalet_in_id".to_string(),
                ))
            }
        };
    }
    Ok(entity_urn)
}

async fn find_entity_urn_sidebar(entity_urn: &WebElement) -> Result<String, CustomError> {
    let href = entity_urn.attr("href").await?;
    println!("href{:#?}", href);
    match href {
        Some(link) => {
            let result = link.split("profileUrn=urn%3Ali%3Afsd_profile%3A").nth(1);
            println!("result {:#?}", result);

            match result {
                Some(result) => match result.split('&').next() {
                    Some(result) => return Ok(result.to_string()),
                    None => {
                        return Err(CustomError::ButtonNotFound(
                            "find_entity_urn_sidebar result not found".to_string(),
                        ))
                    }
                },
                None => {
                    return Err(CustomError::ButtonNotFound(
                        "find_entity_urn_sidebar result not found".to_string(),
                    ))
                }
            }
        }
        None => {
            return Err(CustomError::ButtonNotFound(
                "find_entity_urn_sidebar result not found".to_string(),
            ))
        }
    };
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
