use crate::{
    actions::start_browser::start_browser, actions::wait::wait, structs::candidate::Candidate,
    structs::entry::EntrySendConnection, structs::error::CustomError,
};

use crate::structs::browser::BrowserInit;

pub async fn withdraw_pending(entry: EntrySendConnection) -> Result<(), CustomError> {
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
        .wait_for_selector_builder("input[class=search-global-typeahead__input]")
        .wait_for_selector()
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
    wait(3, 15); // random delay

    browser
        .page
        .wait_for_selector_builder("div.pv-top-card-v2-ctas"); // wait until the block with buttons is loaded

    wait(4, 6);

    let block = match browser
        .page
        .query_selector("div.pv-top-card-v2-ctas")
        .await?
    {
        Some(block) => block,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Block not found/Withdraw".to_string(),
            ));
        }
    };
    let button_selector = r#"button:has-text("Pending")"#;
    let pending_button = match block.query_selector(button_selector).await? {
        Some(button) => button,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Pending button not found/Withdraw".to_string(),
            ));
        }
    };
    pending_button.click_builder().click().await?;
    wait(2, 5);

    // let button_selector = r#"button:has-text("Withdraw")"#;
    //let withdraw_button = block.query_selector(button_selector).await?;
    //withdraw_button.unwrap().click_builder().click().await?;
    let withdraw_selector ="button[class='artdeco-button artdeco-button--2 artdeco-button--primary ember-view artdeco-modal__confirm-dialog-btn']";
    let withdraw_button = match browser.page.query_selector(withdraw_selector).await? {
        Some(button) => button,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Withdraw button in popup not found/Withdraw".to_string(),
            ));
        }
    };

    withdraw_button.click_builder().click().await?;

    browser.page.close(Some(false)).await?; // close page
    browser.browser.close().await?; // close browser

    Ok(())
}
