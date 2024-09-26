use scraper::{Html, Selector};

use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::scrap_recruiter_search::check_recruiter_cookie;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendInmail;
use crate::structs::error::CustomError;
use thirtyfour::{By, WebDriver};
use tracing::instrument;
#[instrument]
pub async fn send_inmails(entry: EntrySendInmail) -> Result<String, CustomError> {
    let message_text = entry
        .message
        .clone()
        .chars()
        .filter(|&c| c as u32 <= 0xFFFF)
        .collect();
    let candidate = Candidate::new(entry.fullname.clone(), entry.linkedin.clone(), message_text);
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
    let result = inmail(&browser, &candidate, &entry.subject).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                text.as_str(),
                &entry.message_id,
                "Send Inmails",
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
                &entry.message_id,
                "Send Inmails",
            )
            .await?;
            return Err(error);
        }
    }
}

pub async fn inmail(
    browser: &WebDriver,
    candidate: &Candidate,
    subject: &str,
) -> Result<String, CustomError> {
    let file_name = "null";
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
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Inmail_regular".to_string(),
                ));
            }
            x += 1;
        }
        wait(1, 3);
    }
    wait(7, 15); // random delay
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
    if main_container.is_err() {
        return Err(CustomError::ButtonNotFound(
            "Main container not found/Send Inmails".to_string(),
        ));
    }
    let entity_urn = find_entity_urn(&main_container.unwrap().inner_html().await?);
    println!("entity_urn {:?}", entity_urn);
    const MAIN_BOX: &str = "main.scaffold-layout__main";
    let main_box = browser.find(By::Css(MAIN_BOX)).await?;

    const MORE_BUTTON: &str =
        "button.artdeco-dropdown__trigger.artdeco-dropdown__trigger--placement-bottom.ember-view.pvs-profile-actions__action.artdeco-button.artdeco-button--secondary.artdeco-button--muted.artdeco-button--2";
    const MORE_BUTTON_ANOTHER: &str = "div.artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-left.ember-view:has(>button[aria-label='More actions'].artdeco-dropdown__trigger):nth-child(3)";
    let more_option = main_box.find(By::Css(MORE_BUTTON)).await;
    let more_option_another = main_box.find(By::Css(MORE_BUTTON_ANOTHER)).await;
    let more_option = match more_option {
        Ok(option) => option,
        Err(_s) => match more_option_another {
            Ok(option) => option,
            Err(_s) => {
                return Err(CustomError::ButtonNotFound(
                    "More button not found".to_string(),
                ));
            }
        },
    };
    match more_option.click().await {
        Ok(_) => (),
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "More button is not clickable".to_string(),
            ));
        }
    };
    wait(2, 3);
    // ("entity_urn: {:?}", entity_urn);
    const VIEW_IN_RECRUITER: &str = "button[class='artdeco-button artdeco-button--2 artdeco-button--secondary ember-view pvs-profile-actions__action']";
    const VIEW_IN_RECRUITER_DROPDOWN: &str = "(//*[contains(@class, 'artdeco-dropdown__item') and contains(@class, 'ember-view') and contains(@class, 'full-width') and contains(@class, 'display-flex') and contains(@class, 'align-items-center') and contains(@aria-label, 'profile in Recruiter')])[2]";
    if entity_urn.is_some() {
        let url = format!(
            "https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER",
            entity_urn.unwrap()
        );
        let mut _go_to = browser.goto(&url).await;
        let mut x = 0;
        if go_to.is_err() {
            while x <= 3 {
                wait(3, 6);
                let build = browser.goto(&url).await;
                if build.is_ok() {
                    _go_to = build; //_go_to never read, is there some point for it?
                    break;
                } else if build.is_err() && x == 3 {
                    wait(3, 6);
                    return Err(CustomError::ButtonNotFound(
                        "Candidate Recruiter page is not loading/Inmail".to_string(),
                    )); // if error means page is not loading
                }
                x += 1;
            }
        }
    } else {
        let view_in_recruiter_button = browser.find(By::Css(VIEW_IN_RECRUITER)).await;
        match view_in_recruiter_button {
            Ok(button) => {
                button.click().await?;
            }
            Err(_) => match browser.find(By::XPath(VIEW_IN_RECRUITER_DROPDOWN)).await {
                Ok(button) => button.click().await?,
                Err(_e) => {
                    return Err(CustomError::ButtonNotFound(
                        "View in recruiter button is not found".to_string(),
                    ));
                }
            },
        }
        wait(6, 10);
    }
    wait(10, 15);

    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::RecruiterSessionCookieExpired);
        }
    }
    wait(6, 16);

    wait(2, 4);
    const SEND_BUTTON: &str = "button[class='artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--2 artdeco-button--tertiary ember-view profile-item-actions__item']";
    const DISABLED_BUTTON:&str = "button.artdeco-button.artdeco-button--circle.artdeco-button--muted.artdeco-button--2.artdeco-button--tertiary.ember-view.profile-item-actions__item:disabled";

    let disabled_button = browser.find(By::Css(DISABLED_BUTTON)).await;
    if disabled_button.is_ok() {
        return Ok("Sending Inmails is disabled for this account".to_string());
    };

    let send_button = browser.find(By::Css(SEND_BUTTON)).await;

    match send_button {
        Ok(button) => {
            button.click().await?; // hover on search input
            wait(5, 7); // random delay
        }
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Send Inmails".to_string(),
            ));
        }
    };

    if file_name != "null" {
        return Err(CustomError::ButtonNotFound(
            "Inmail file not send".to_string(),
        ));
    }
    const REMOVE_AI_TEXT: &str =
        "button.compose-textarea-ghost-cta__button.t-14.t-black--light:not([aria-label])";

    let remove_ai_text = browser.find(By::Css(REMOVE_AI_TEXT)).await;

    if remove_ai_text.is_ok() {
        let button = remove_ai_text.unwrap();
        button.click().await?;
        wait(2, 3);
    }
    const SUBJECT_INPUT: &str = "input[class='compose-subject__input']";

    let subject_input = browser.find(By::Css(SUBJECT_INPUT)).await;

    match subject_input {
        Ok(input) => {
            input.click().await?;
            wait(2, 5);
            input.send_keys(subject).await?;
        }
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Subject input in recruiter is not visible".to_string(),
            ));
        }
    };
    wait(2, 5);
    const TEXT_INPUT: &str = "div[class='ql-editor ql-blank']";

    let text_input = browser.find(By::Css(TEXT_INPUT)).await;

    match text_input {
        Ok(input) => {
            input.click().await?; // click on search input
            wait(2, 5);
            input.send_keys(&candidate.message).await?;
        }
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Text input in recruiter is not visible".to_string(),
            ));
        }
    };
    const FIRST_BUTTON: &str =
        "button[class='artdeco-button artdeco-button--2 artdeco-button--primary ember-view']";
    const SECOND_BUTTON: &str =
        "button[class='artdeco-button artdeco-button--2 artdeco-button--primary ember-view']";

    let first_button = browser.find(By::Css(FIRST_BUTTON)).await;

    //checking between 2 possible button variations

    if let Ok(button) = first_button {
        wait(1, 4);
        button.click().await?; // click on search input
        wait(2, 5);
    } else {
        let second_button = browser.find(By::Css(SECOND_BUTTON)).await;

        if let Ok(button) = second_button {
            wait(1, 4);
            button.click().await?;
            wait(2, 5);
        } else {
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Text".to_string(),
            ));
        }
    }

    wait(2, 4);
    Ok("Inmail was sent".to_string())
}

fn find_entity_urn(html: &str) -> Option<String> {
    let link_selector = Selector::parse("a").unwrap();
    let document = scraper::Html::parse_document(&html);
    let mut entity_urn = String::new();

    for link in document.select(&link_selector) {
        let href = link.value().attr("href").unwrap_or_default();
        if href.contains("profileUrn=") {
            // First attempt: Split on "?profileUrn=urn%3Ali%3Afsd_profile%3A"
            let parts: Vec<&str> = href
                .split("?profileUrn=urn%3Ali%3Afsd_profile%3A")
                .collect();
            if parts.len() > 1 {
                entity_urn = parts[1].split('&').collect::<Vec<&str>>()[0].to_string();
                if entity_urn.is_empty() {
                    let parts: Vec<&str> = href
                        .split("?profileUrn=urn%3Ali%3Afs_normalized_profile%3A")
                        .collect();
                    if parts.len() > 1 {
                        entity_urn = parts[1].split('&').collect::<Vec<&str>>()[0].to_string();
                    }
                }
            } else {
                // Additional if branch: Split on "&profileUrn=urn%3Ali%3Afsd_profile%3A"
                let parts: Vec<&str> = href
                    .split("&profileUrn=urn%3Ali%3Afsd_profile%3A")
                    .collect();
                if parts.len() > 1 {
                    entity_urn = parts[1].split('&').collect::<Vec<&str>>()[0].to_string();
                    if entity_urn.is_empty() {
                        let parts: Vec<&str> = href
                            .split("&profileUrn=urn%3Ali%3Afs_normalized_profile%3A")
                            .collect();
                        if parts.len() > 1 {
                            entity_urn = parts[1].split('&').collect::<Vec<&str>>()[0].to_string();
                        }
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
