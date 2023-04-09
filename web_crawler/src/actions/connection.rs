use playwright::api::{Cookie, Page, ProxySettings};
use playwright::Playwright;
use std::collections::HashMap;

use crate::structs::entry::Entry;
use crate::structs::user::User;

use crate::actions::wait::wait;
use crate::structs::candidate::Candidate;

pub async fn connection(entry: Entry) -> Result<(), playwright::Error> {
    println!("1");
    let candidate = Candidate::new(entry.fullname, entry.linkedin, entry.message);
    println!("2");
    println!("3");
    let user = User::new(entry.user_agent, entry.session_cookie, entry.user_id);
    println!("4");
    //proxy settings, TODO add variables instead of fixed values
    let proxy = ProxySettings {
        server: entry.ip,
        username: Some(entry.username),
        password: Some(entry.password),
        bypass: None,
    };
    println!("5");
    // start browser
    let playwright = Playwright::initialize().await?;
    println!("6");
    playwright.prepare()?; // Install browsers
    println!("7");
    let chromium = playwright.chromium();
    println!("8");

    let browser = chromium
        .launcher()
        .proxy(proxy)
        .headless(false)
        .launch()
        .await?;
    println!("9");

    //headers, TODO add variable for User-Agent
    let context = browser.context_builder().build().await?;
    println!("10");
    let page_1 = context.new_page().await;

    match page_1 {
        Ok(_) => {
            println!("page is ok");
        }
        Err(error) => {
            println!("page is not ok {}", error);
        }
    }

    let page = context.new_page().await?;
    println!("11");

    let mut headers = HashMap::new();

    headers.insert("User-Agent".to_string(), user.user_agent);

    context.set_extra_http_headers(headers).await?;

    //it appears only if you visit the target url, otherwise cookie won't show
    let cookie = Cookie::with_url(
        "li_at",
        user.session_cookie.as_str(),
        "https://.www.linkedin.com",
    );

    context.add_cookies(&[cookie]).await?;

    //TODO add variable for the url
    let url = page
        .goto_builder("https://www.linkedin.com/feed/")
        .goto()
        .await;
    if url.is_err() {
        wait(3, 15); // random delay
        browser.close().await?;
        return Err(playwright::Error::Channel); // if url is not valid, means proxy is not valid or internet connection is not working
    }

    wait(3, 15);
    page.wait_for_selector_builder("input[class=search-global-typeahead__input]");

    //end browser

    let search_input = page
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
            browser.close().await?; // close browser
            return Err(playwright::Error::InitializationError);
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    // go to candidate page
    page.goto_builder(candidate.linkedin.as_str())
        .goto()
        .await?;
    wait(3, 15); // random delay
                 //check if connect button is present
    let connect_button = find_button(&page).await;
    match connect_button {
        Ok(_) => message(&page, candidate.message.as_str()).await?,
        Err(_) => {
            wait(2, 6); // random delay
            browser.close().await?;
            return Err(playwright::Error::ObjectNotFound);
        }
    }

    wait(3, 15); // random delay; // add delay before closing the browser to check things

    browser.close().await?;
    Ok(())
}

async fn find_button(page: &Page) -> Result<(), playwright::Error> {
    // find the block with buttons
    let block = page
        .query_selector("div[class=pv-top-card-v2-ctas]")
        .await?;
    match block {
        Some(_) => (),
        None => return Err(playwright::Error::ObjectNotFound),
    }
    // find button more actions
    let more = block
        .as_ref()
        .unwrap()
        .query_selector("button[aria-label='More actions']")
        .await?;
    match more {
        Some(more) => more.click_builder().click().await?, //click on button more actions
        None => return Err(playwright::Error::ObjectNotFound),
    }

    wait(1, 3); // random delay
                //find button connect
    let connect = block
        .unwrap()
        .query_selector("li-icon[type=connect]")
        .await?;
    match connect {
        Some(connect) => connect.click_builder().click().await?, //click on button connect
        None => return Err(playwright::Error::ObjectNotFound),
    }

    //check if popup to choose "How do you know" appeares
    let popup_how = page.query_selector("button[aria-label='Other']").await?;

    match popup_how {
        Some(popup_how) => {
            popup_how.click_builder().click().await?; // click on button "Other"
                                                      // click on button "Connect"
            let connect = page.query_selector("button[aria-label='Connect']").await?;
            match connect {
                Some(connect) => connect.click_builder().click().await?,
                None => return Err(playwright::Error::ObjectNotFound),
            }
        }
        None => (),
    };

    Ok(())
}

async fn message(page: &Page, message: &str) -> Result<(), playwright::Error> {
    //press button add note
    let add_note = page
        .query_selector("button[aria-label='Add a note']")
        .await?;
    match add_note {
        Some(add_note) => add_note.click_builder().click().await?, // click on button "Other"
        None => return Err(playwright::Error::ObjectNotFound),
    };
    //find input for note
    let text_input = page.query_selector("textarea[id=custom-message]").await?;
    match text_input {
        Some(text_input) => {
            text_input.hover_builder(); // hover on input for note
            wait(1, 3); // random delay
            text_input.focus().await?; // focus on input for note
            wait(1, 2); // random delay
            text_input.fill_builder(message).fill().await?; // fill input for note;
        }
        None => return Err(playwright::Error::ObjectNotFound),
    };

    wait(1, 3); // random delay
                //press button send
    let send = page.query_selector("button[aria-label='Send now']").await?;
    match send {
        Some(send) => send.click_builder().click().await?, // click on button "Send"
        None => return Err(playwright::Error::ObjectNotFound),
    };

    Ok(())
}
