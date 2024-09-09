use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use thirtyfour::{By, Key, WebDriver};
use tracing::info;
pub async fn connection(entry: EntrySendConnection) -> Result<String, CustomError> {
    info!("Sending connection request to {}", entry.fullname);

    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );
    let user_id = entry.user_id.clone();

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
                    "Candidate page is not loading/Send_connection",
                    "Send connection",
                )
                .await?;

                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Send_connection_message".to_string(),
                ));
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
            wait(1, 3);
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Session cookie expired",
                "Send connection",
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
            "Main container not found/Send_connection_message",
            "Send connection",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Main container not found/Send regular message".to_string(),
        ));
    }
    const PAGE_NOT_FOUND: &str = "header[class='not-found__header not-found__container']";
    let page_not_found = browser.find(By::Css(PAGE_NOT_FOUND)).await;

    match page_not_found {
        Ok(_) => {
            wait(1, 3);
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Page does not exist",
                "Send connection",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Page does not exist".to_string(),
            ));
        }
        Err(_) => (),
    };
    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Session cookie expired",
                "Send connection",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }
    }
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
                let screenshot = browser.screenshot_as_png().await?;
                browser.quit().await?;
                send_screenshot(
                    screenshot,
                    &browser_info.user_id,
                    "More button not found",
                    "Send connection",
                )
                .await?;
                return Err(CustomError::ButtonNotFound(
                    "More button not found".to_string(),
                ));
            }
        },
    };
    more_option.click().await?;
    const IN_CONNECTION_POOL: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Remove your connection']";
    let in_connection_pool = browser.find(By::Css(IN_CONNECTION_POOL)).await;
    if in_connection_pool.is_ok() {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Candidate in connection pool",
            "Send connection",
        )
        .await?;
        return Ok("Candidate in connection pool".to_string());
    }

    const PENDING_ON_THE_PAGE: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--secondary.ember-view.pvs-profile-actions__action[aria-label*='Pending']";
    const PENDING_DROPDOWN: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Pending']";

    let pending_on_the_page = browser.find(By::Css(PENDING_ON_THE_PAGE)).await;
    let pending_dropdown = browser.find(By::Css(PENDING_DROPDOWN)).await;

    if pending_on_the_page.is_ok() || pending_dropdown.is_ok() {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Connection pending",
            "Send connection",
        )
        .await?;
        return Ok("Connection pending".to_string());
    }

    const CONNECT_ON_THE_PAGE:&str = "button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.pvs-profile-actions__action[aria-label*='connect']";
    const CONNECT_DROPDOWN:&str = "
(//div[contains(@class, 'artdeco-dropdown__item') and contains(@class, 'artdeco-dropdown__item--is-dropdown') and contains(@class, 'ember-view') and contains(@class, 'full-width') and contains(@class, 'display-flex') and contains(@class, 'align-items-center') and contains(@aria-label, 'connect')])[2]
";

    let connect_on_the_page = browser.find(By::Css(CONNECT_ON_THE_PAGE)).await;
    let connect_dropdown = browser.find(By::XPath(CONNECT_DROPDOWN)).await;
    let connect_button = if let Ok(button) = connect_on_the_page {
        button
    } else if let Ok(button) = connect_dropdown {
        button
    } else {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Connection button missing",
            "Send connection",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Connection button missing".to_string(),
        ));
    };

    connect_button.click().await?;

    wait(2, 3);
    //check if popup to choose "How do you know"
    const POPUP_HOW: &str = "button[aria-label='Other']";
    let popup_how = browser.find(By::Css(POPUP_HOW)).await;

    match popup_how {
        Ok(popup_how) => {
            popup_how.click().await?;
            wait(1, 3);
            const POPUP_HOW_CONNECT: &str = "button[aria-label='Connect']";
            let connect = browser.find(By::Css(POPUP_HOW_CONNECT)).await;

            match connect {
                Ok(connect) => connect.click().await?,
                Err(_s) => {
                    let screenshot = browser.screenshot_as_png().await?;
                    browser.quit().await?;
                    send_screenshot(
                        screenshot,
                        &browser_info.user_id,
                        "Connect button in popup_how is not found",
                        "Send connection",
                    )
                    .await?;
                    return Err(CustomError::ButtonNotFound(
                        "Connect button in popup_how is not found".to_string(),
                    ));
                }
            }
        }
        Err(_s) => (),
    };
    const EMAIL_NEEDED: &str = "label[for=email]";
    let email_needed = browser.find(By::Css(EMAIL_NEEDED)).await;

    match email_needed {
        Ok(_) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Email needed",
                "Send connection",
            )
            .await?;
            return Ok("Email needed".to_string());
        }
        Err(_s) => (),
    };
    let adding_message = message(&browser, candidate.message.as_str(), &user_id).await;

    if let Err(error) = adding_message {
        browser.quit().await?;
        return Err(error);
    }

    wait(4, 8);

    let pending_on_the_page = browser.find(By::Css(PENDING_ON_THE_PAGE)).await;
    let pending_dropdown = browser.find(By::Css(PENDING_DROPDOWN)).await;

    if pending_on_the_page.is_ok() || pending_dropdown.is_ok() {
        println!("Pending found");
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Connection pending",
            "Send connection",
        )
        .await?;
        return Ok("Connection was sent".to_string());
    }

    wait(3, 7);
    const CONNNECTION_LIMIT: &str =
        "div[class='artdeco-modal artdeco-modal--layer-default ip-fuse-limit-alert']";
    let connection_limit = browser.find(By::Css(CONNNECTION_LIMIT)).await;

    match connection_limit {
        Ok(_) => {
            wait(1, 5);
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Connection limit",
                "Send connection",
            )
            .await?;
            return Err(CustomError::ConnectionLimit);
        }
        Err(_s) => (),
    };
    wait(3, 15); // random delay; // add delay before closing the browser to check things

    browser.quit().await?;
    Ok("Connection was sent".to_string())
}
async fn message(browser: &WebDriver, message: &str, user_id: &str) -> Result<(), CustomError> {
    //press button add note
    wait(5, 7);
    const ADD_NOTE: &str = "button.artdeco-button.artdeco-button--muted.artdeco-button--2.artdeco-button--secondary.ember-view.mr1";
    let add_note = browser.find(By::Css(ADD_NOTE)).await;
    let add_note = match add_note {
        Ok(add_note) => add_note,
        Err(_s) => {
            wait(1, 5);
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Add note button not found",
                "Send connection",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Add note button not found".to_string(),
            ));
        }
    };
    add_note.click().await?;
    info!("Filling in the message field");
    wait(12, 15);
    const TEXT_INPUT: &str = "textarea[id=custom-message]";
    let text_input = browser.find(By::Css(TEXT_INPUT)).await;
    match text_input {
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
            input.send_keys(message).await?; // fill input for note;
        }
        Err(_) => {
            wait(1, 5);
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Text input not found",
                "Send connection",
            )
            .await?;

            return Err(CustomError::ButtonNotFound(
                "Text input not found".to_string(),
            ));
        }
    };
    wait(1, 3); // random delay
    const SEND_BUTTON: &str =
        "button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.ml1";
    let send_button = browser.find(By::Css(SEND_BUTTON)).await;
    match send_button {
        Ok(button) => {
            button.click().await?;
            wait(2, 3)
        }
        Err(_) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Send button in the connection popup is not found",
                "Send connection",
            )
            .await?;

            return Err(CustomError::ButtonNotFound(
                "Send button in the connection popup is not found".to_string(),
            ));
        }
    };
    return Ok(()); // return Ok
}
