use crate::structs::entry::Entry;

use crate::actions::wait::wait;
use crate::structs::candidate::{Candidate};

use super::start_browser::start_browser;



pub async fn send_message(entry: Entry) -> Result<(), playwright::Error> {

let candidate = Candidate::new(entry.fullname.clone(), entry.linkedin.clone(), entry.message.clone());

let browser = start_browser(entry).await?;

let search_input = browser.page
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
            browser.browser.close().await?; // close browser
            return Err(playwright::Error::InitializationError);
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

// go to candidate page
browser.page.goto_builder(candidate.linkedin.as_str())
.goto()
.await?;
wait(3, 15); // random delay

browser.page.wait_for_selector_builder("div.pv-top-card-v2-ctas"); // wait until the block with buttons is loaded

let message_button = browser.page
    .query_selector("button.entry-point.pvs-profile-actions__action")
    .await;



let message_button = match message_button {
    Ok(button) => match button {
        Some(button) => button,
        None => { 
            wait(1, 5); // random delay
            browser.browser.close().await?;
            return Err(playwright::Error::ObjectNotFound)
                }, // means there is no message button
    },
    Err(_) => {
            wait(1, 5); // random delay
            browser.browser.close().await?;
            return Err(playwright::Error::Timeout)
              },
            
};

message_button.hover_builder(); // hover on search input
wait(1, 4); // random delay
message_button.click_builder().click().await?; // click on search input
wait(2, 5); // random delay

let regular_input = browser.page
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
    },
    None => {
        wait(1, 5); // random delay   
        browser.browser.close().await?;
        return Err(playwright::Error::InvalidParams)
        } // means you can't send message to this profile
}

let send = browser.page
    .query_selector("button.msg-form__send-button.artdeco-button.artdeco-button--1")
    .await?;

match send {
    Some(send) => {
        send.hover_builder(); // hover on search input
        wait(1, 4); // random delay
        send.click_builder().click().await?; // click on search input
        wait(2, 5); // random delay
    },
    None => {
        wait(1, 5); // random delay
        browser.browser.close().await?;
        return Err(playwright::Error::NotObject)
        } // means you can't send message to this profile
   
}

browser.browser.close().await?;
Ok(())
}