use scraper::{Html, Selector};

use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendInmail;
use crate::structs::error::CustomError;
use playwright::api::Page;
use tracing::instrument;
use tracing::span;
use tracing::Level;

#[instrument]
pub async fn send_inmails(entry: EntrySendInmail) -> Result<(), CustomError> {
    let span = span!(Level::DEBUG, "sub_span_name {}", entry.message_id);
    let _enter = span.enter();
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );

    let subject = entry.subject.clone();

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

    let search_input = browser
        .page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;

    wait(3, 15); // random delay
                 //focus on search input and fill it with text
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
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    // go to candidate page
    let mut go_to = browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser
                .page
                .goto_builder(candidate.linkedin.as_str())
                .goto()
                .await;
            if build.is_ok() {
                go_to = build;
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Inmail_regular".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            //// ("retrying to load page")
        }
        wait(1, 3);
    }

    wait(3, 15); // random delay
                 //check if View in recruiter is present
                 /*
                   let view_button = browser
                   .page
                   .query_selector("button[class='artdeco-button artdeco-button--2 artdeco-button--secondary ember-view pvs-profile-actions__action']")
                   .await?;

                   match view_button {
                    Some(view_button) => {
                       view_button.hover_builder(); // hover on search input
                        wait(1, 4); // random delay
                        view_button.click_builder().click().await?; // click on search input
                        wait(2, 5); // random delay
                    }
                    None => {
                        wait(1, 5); // random delay
                        browser.page.close(Some(false)).await?;
                        browser.browser.close().await?; // close browser
                        return Err(CustomError::ButtonNotFound("View in recruiter button is not visible".to_string()));
                    } // if search input is not found, means page was not loaded and sessuion cookie is not valid
                 };
                 */

    let entity_urn = find_entity_urn(&browser.page).await?;

    // ("entity_urn: {:?}", entity_urn);
    const VIEW_IN_RECRUITER: &str = "button[class='artdeco-button artdeco-button--2 artdeco-button--secondary ember-view pvs-profile-actions__action']";
    if entity_urn.is_some() {
        let url = format!(
            "https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER",
            entity_urn.unwrap()
        );
        // go to candidate page777
        let mut _go_to = browser.page.goto_builder(url.as_str()).goto().await;
        let mut x = 0;
        if go_to.is_err() {
            while x <= 3 {
                wait(3, 6);
                let build = browser.page.goto_builder(url.as_str()).goto().await;
                if build.is_ok() {
                    _go_to = build; //_go_to never read, is there some point for it?
                    break;
                } else if build.is_err() && x == 3 {
                    wait(3, 6);
                    browser.page.close(Some(false)).await?;
                    browser.browser.close().await?; // close browser
                    return Err(CustomError::ButtonNotFound(
                        "Candidate Recruiter page is not loading/Inmail".to_string(),
                    )); // if error means page is not loading
                }
                x += 1;
                //// ("retrying to load page")
            }
        }
    } else {
        let view_in_recruiter_button = browser.page.query_selector(VIEW_IN_RECRUITER).await?;
        match view_in_recruiter_button {
            Some(button) => button.click_builder().click().await?,
            None => {
                return Err(CustomError::ButtonNotFound(
                    "Vier in recruiter button is not found".to_string(),
                ))
            }
        }
        wait(6, 10);
        let opened_pages = browser.context.pages().unwrap();
        let url = opened_pages.get(1).unwrap().url()?;
        let _result = browser.page.goto_builder(url.as_str()).goto().await;
        //browser.page.close(Some(false)).await?;
    }
    wait(10, 15);

    let nav_bar = browser
        .page
        .query_selector("div[class='global-nav__right']")
        .await?;

    //wait(10000000, 100000000000);
    match &nav_bar {
        Some(_) => (),
        None => {
            wait(1, 3);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::RecruiterSessionCookieExpired); // if error when session cookie expired
        }
    }

    wait(6, 16);
    /*
        const PROFILE_OLD_BLOCK: &str = "div[class='topcard-condensed__content-top topcard-condensed__content-top--profile-size-7']";
        const PROFILE_NEW_BLOCK: &str =

        let profile_block = if browser
            .page
            .query_selector(PROFILE_OLD_BLOCK)
            .await?
            .is_some()
        {
            browser.page.query_selector(PROFILE_OLD_BLOCK).await?
        } else {
            browser.page.query_selector(PROFILE_NEW_BLOCK).await?
        };
        match &profile_block {
            Some(_) => (),
            None => {
                wait(1, 3);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?;
                return Err(CustomError::ProfileNotFound);
            }
        }
    */
    wait(2, 4);

    let send_button = browser
.page
.query_selector("button[class='artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--2 artdeco-button--tertiary ember-view profile-item-actions__item']")
.await?;

    match send_button {
        Some(button) => {
            button.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            button.click_builder().click().await?; // click on search input
            wait(5, 7); // random delay
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Page".to_string(),
            ));
        }
    };

    if entry.file_name != "null" {
        return Err(CustomError::ButtonNotFound(
            "Inmail file not send".to_string(),
        ));
    }
    let remove_ai_text = browser
        .page
        .query_selector("button[class='compose-textarea-ghost-cta__button t-14 t-black--light']")
        .await?;

    if remove_ai_text.is_some() {
        let button = remove_ai_text.unwrap();
        button.hover_builder();
        wait(1, 2);
        button.click_builder().click().await?;
        wait(2, 3);
    }
    let subject_input = browser
        .page
        .query_selector("input[class='compose-subject__input']")
        .await?;

    match subject_input {
        Some(input) => {
            input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            input.fill_builder(subject.as_str()).fill().await?; // fill input for note;
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Subject input in recruiter is not visible".to_string(),
            ));
        }
    };
    wait(2, 5);

    let text_input = browser
        .page
        .query_selector("textarea[class='compose-textarea__textarea']")
        .await?;

    match text_input {
        Some(input) => {
            input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            input
                .fill_builder(candidate.message.as_str())
                .fill()
                .await?; // fill input for note;
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Subject input in recruiter is not visible".to_string(),
            ));
        }
    };
    //checking between 2 possible button variations
    let first_button = browser
        .page
        .query_selector(
            "button[class='artdeco-button artdeco-button--2 artdeco-button--primary ember-view']",
        )
        .await?;

    if let Some(button) = first_button {
        // Do actions with the first button found
        // ("First button found.");
        button.hover_builder(); // hover on search input
        wait(1, 4); // random delay
        button.click_builder().click().await?; // click on search input
        wait(2, 5); // random delay
    } else {
        let second_button = browser
            .page
            .query_selector(
                "button[class='msg-cmpt__button--small compose-actions__submit-button']",
            )
            .await?;

        if let Some(button) = second_button {
            button.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            button.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
        } else {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Send button in recruiter is not visible/Text".to_string(),
            ));
        }
    }

    /*
        match send_button {
            Some(button) => {
                button.hover_builder(); // hover on search input
                wait(1, 4); // random delay
                button.click_builder().click().await?; // click on search input
                wait(2, 5); // random delay
            }
            None => {
                wait(1, 5); // random delay
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Send button in recruiter is not visible/Text".to_string(),
                ));
            }
        };
    */
    wait(2, 4);
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    drop(_enter);
    Ok(())
}

async fn find_entity_urn(page: &Page) -> Result<Option<String>, playwright::Error> {
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
        entity_urn = match print_elements_with_datalet_in_id(document.html().as_str()) {
            Some(urn) => urn,
            None => return Ok(None),
        };
    }
    Ok(Some(entity_urn))
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
