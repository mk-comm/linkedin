use playwright::api::Page;
use crate::structs::browser::BrowserInit;

use crate::actions::start_browser::start_browser;
use crate::structs::entry::Entry;
use crate::structs::error::CustomError;
use crate::actions::wait::wait;
use crate::structs::candidate::Candidate;

pub async fn connection(entry: Entry) -> Result<(), CustomError> {
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
        };

   // return Err(CustomError::ButtonNotFound("Connect button not found".to_string()));

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
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::SessionCookieExpired);
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    // go to candidate page
    let go_to = browser
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
                    break;
                } else if build.is_err() && x == 3 {
                    wait(3, 6);
                    browser.page.close(Some(false)).await?;
                    browser.browser.close().await?; // close browser
                    return Err(CustomError::ButtonNotFound("Candidate page is not loading/Connection".to_string())); // if error means page is not loading
                }
                x += 1;
                println!("retrying to load page")
            }
            wait(1, 3);
        }


    wait(3, 15); // random delay
                 //check if connect button is present

    let block = browser
        .page
        .query_selector("div.pvs-profile-actions")
        .await?
        .unwrap();

    let connect_button = block.query_selector("li-icon[type=connect]").await?;

    let more = block
        .query_selector("button[aria-label='More actions']")
        .await?
        .unwrap();

    more.hover_builder();
    wait(1, 4);
    more.click_builder().click().await?;
    wait(2, 5);

    match connect_button {
        Some(button) => {
            button.hover_builder();
            wait(1, 4);
            button.click_builder().click().await?;
            wait(2, 5);
        }
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound("Connect button not found".to_string()));
        }
    }

    
    
    //check if popup to choose "How do you know" appeares
    let popup_how = browser
    .page
    .query_selector("button[aria-label='Other']")
    .await?;

    match popup_how {
    Some(popup_how) => {
        popup_how.click_builder().click().await?; // click on button "Other"
        
        let connect = browser
        .page
        .query_selector("button[aria-label='Connect']")
        .await?;
    match connect {
        Some(connect) => connect.click_builder().click().await?,
                None => {
                    wait(1, 5);
                    browser.page.close(Some(false)).await?;
                    browser.browser.close().await?;
                    return Err(playwright::Error::InvalidParams.into())
                },
            }
        }
        None => (),
    };
    

    let email_needed = browser.page.query_selector("label[for=email]").await?;

    match email_needed {
        Some(_) => {
                wait(1, 5);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?;
                return Err(CustomError::EmailNeeded);   
        }
        None => (),
    };
        
    
    //artdeco-modal artdeco-modal--layer-default send-invite  
    message(&browser.page, candidate.message.as_str()).await?;
    wait(3, 7);
    let connection_limit = browser.page.query_selector("div[class='artdeco-modal artdeco-modal--layer-default ip-fuse-limit-alert']").await?;

    match connection_limit {
        Some(_) => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ConnectionLimit);
        }
        None => (),
    };
    wait(3, 15); // random delay; // add delay before closing the browser to check things

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;
    Ok(())
}

async fn message(page: &Page, message: &str) -> Result<(), playwright::Error> {
    //press button add note
    let add_note = page
        .query_selector("button[aria-label='Add a note']")
        .await?;
    match add_note {
        Some(add_note) => add_note.click_builder().click().await?, // click on button "Other"
        None => return Err(playwright::Error::InvalidParams),
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
        None => return Err(playwright::Error::InvalidParams),
    };

    wait(1, 3); // random delay
                //press button send
    let send = page.query_selector("button[aria-label='Send now']").await?;
    match send {
        Some(send) => send.click_builder().click().await?, // click on button "Send"
        None => return Err(playwright::Error::InvalidParams),
    };

    Ok(())
}
