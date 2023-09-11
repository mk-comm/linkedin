use crate::{
    actions::start_browser::start_browser, actions::wait::wait, structs::candidate::Candidate,
    structs::entry::Entry, structs::error::CustomError,
};

use crate::structs::browser::BrowserInit;

pub async fn withdraw(entry: Entry) -> Result<(), CustomError> {
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
        recruiter_session_cookie: Some(entry.recruiter_session_cookie),
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
    //"artdeco-button.artdeco-button--muted.artdeco-button--2.artdeco-button--secondary.ember-view.pvs-profile-actions__action"
    //"artdeco-dropdown.artdeco-dropdown--placement-bottom.artdeco-dropdown--justification-left.ember-view"
    let withdraw_button = block
        .query_selector("button.artdeco-button.artdeco-button--muted.artdeco-button--2.artdeco-button--secondary.ember-view.pvs-profile-actions__action")
        .await?;
    match withdraw_button {
        Some(button) => {
            let icon = button
                .query_selector("li-icon.artdeco-button__icon")
                .await?;
            match icon {
                Some(icon) => {
                    //println!("Icon found");
                    icon.click_builder().click().await?; // click on icon pending
                    wait(1, 3);
                    let withdraw = browser.page.
                        wait_for_selector_builder("button.artdeco-modal__confirm-dialog-btn.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view")
                        .wait_for_selector().await?;
                    match withdraw {
                        Some(withdraw) => {
                            withdraw.click_builder().click().await?; // press withdraw in popup
                            wait(1, 3);
                        }
                        None => {
                            wait(1, 5);
                            browser.page.close(Some(false)).await?;
                            browser.browser.close().await?;
                            return Err(CustomError::ButtonNotFound(
                                "Withdraw icon not found/Withdraw".to_string(),
                            ));
                        }
                    }
                }
                None => {
                    //println!("Icon not found");
                    button.click_builder().click().await?;
                    wait(1, 3);
                    let dropdown = block
                        .query_selector("div.pvs-overflow-actions-dropdown__content.artdeco-dropdown__content.artdeco-dropdown--is-dropdown-element.artdeco-dropdown__content--justification-left.artdeco-dropdown__content--placement-bottom.ember-view")
                        .await?;
                    if let Some(dropdown) = dropdown {
                        let icon = dropdown
                            .query_selector("li-icon[type=remove-connection]")
                            .await?;
                        match icon {
                            Some(icon) => {
                                icon.hover_builder();
                                wait(1, 3);
                                icon.click_builder().click().await?;
                                wait(1, 3);
                            }
                            None => {
                                wait(1, 3);
                                browser.page.close(Some(false)).await?;
                                browser.browser.close().await?; // close browser
                                return Err(CustomError::ButtonNotFound(
                                    "Dropdown not found/Withdraw".to_string(),
                                ));
                            }
                        };
                    }
                    // press on more
                    // check if withdraw connection
                    // if press withdraw_button
                    // else return no connection to withdraw
                }
            }
        }
        None => {
            return Err(CustomError::ButtonNotFound(
                "Withdown button not found/Withdraw".to_string(),
            ));
        }
    }

    wait(1, 5);
    browser.page.close(Some(false)).await?; // close page
    browser.browser.close().await?; // close browser

    Ok(())
}
