
use crate::structs::entry::Entry;
use scraper::{Html, Selector};
use crate::actions::wait::wait;
use crate::structs::candidate::Candidate;

use super::start_browser::start_browser;

pub async fn send_message(entry: Entry) -> Result<(), playwright::Error> {
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );

    let entity_urn = entry.entity_urn.clone();

    let browser = start_browser(entry).await?;

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
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(playwright::Error::ReceiverClosed);
        } // if search input is not found, means page was not loaded and session cookie is not valid
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
        .query_selector("div.entry-point.pvs-profile-actions__action")
        .await;

    let message_button = match message_button {
        Ok(button) => match button {
            Some(button) => button,
            None => {
                wait(1, 5); // random delay
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?;
                return Err(playwright::Error::ObjectNotFound);
            } // means there is no message button
        },
        Err(_) => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(playwright::Error::ObjectNotFound);
        }
    };

    message_button.hover_builder(); // hover on search input
    wait(1, 4); // random delay
    message_button.click_builder().click().await?; // click on search input
    wait(2, 5); // random delay
    // Picking the right conversation

    // Get the HTML content of the messaging container
    let pick = browser.page.query_selector("aside.msg-overlay-container").await?.unwrap();
    let html = pick.inner_html().await?;
    let conversation_id = find_conversation(html.as_str(), entity_urn.as_str());

    let conversation_select = match browser.page
    .query_selector(format!("div[id='{}']", conversation_id).as_str())
    .await?
{
    Some(conversation) => {
        conversation
    }
    None =>{
    wait(1, 5); // random delay
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    return Err(playwright::Error::NotObject);
    }
}; // select the conversation that matches the entity_urn


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
            return Err(playwright::Error::InvalidParams);
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
            return Err(playwright::Error::ObjectNotFound);
        } // means you can't send message to this profile
    }

    wait(5, 7);
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}

fn find_conversation(html: &str, entity_urn: &str) -> String {
    
    // Parse the HTML content and find the required div
    let document = Html::parse_document(&html);
    let conv_selector = Selector::parse("div.msg-convo-wrapper").unwrap();
    let href_selector = Selector::parse("a[href^='/in/']").unwrap();
   
    //let code = "/in/ACoAADcTjioB4nj57dk1rAQazWKnfNn4AjQKHNc/"; //target URN
    let code = format!("/in/{}/", entity_urn); //target URN
    println!("Code variable: {}", code);
    let mut correct_div = String::new();
    for conv_div in document.select(&conv_selector) {
 
        let id = conv_div.value().attr("id").unwrap();
        println!("{}", id);
        let href_elem = conv_div.select(&href_selector).next().unwrap();
    
        let href = href_elem.value().attr("href").unwrap();
        println!("Href variable: {}", href);
    
        if href == code {
        //println!(", {}", conv_div.inner_html());
            correct_div = id.to_owned();
        //let button = container.query_selector("button[class='msg-form__send-toggle artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--1 artdeco-button--tertiary ember-view']").await?.unwrap();
     //   button.click_builder();
        }
    }

correct_div

}