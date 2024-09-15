use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::structs::browser::BrowserInit;
use crate::{
    actions::wait::wait, structs::candidate::Candidate, structs::entry::EntrySendConnection,
    structs::error::CustomError,
};
use thirtyfour::By;

pub async fn withdraw_pending(entry: EntrySendConnection) -> Result<String, CustomError> {
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

    let go_to = browser.goto(&candidate.linkedin).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(&candidate.linkedin).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                let screenshot = browser.screenshot_as_png().await?;
                browser.quit().await?;
                send_screenshot(
                    screenshot,
                    &browser_info.user_id,
                    "Candidate page is not loading/Withdraw connection",
                    "Withdraw connection",
                )
                .await?;

                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Withdraw connection".to_string(),
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
                "Withdraw connection",
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
            "Withdraw connection",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Main container not found/Withdraw connection".to_string(),
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
                "Withdraw connection",
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
                "Withdraw connection",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const MAIN_BOX: &str = "main.scaffold-layout__main";
    let main_box = browser.find(By::Css(MAIN_BOX)).await?; // wait until the block with buttons is loaded
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
                    "Withdraw connection",
                )
                .await?;
                return Err(CustomError::ButtonNotFound(
                    "More button not found".to_string(),
                ));
            }
        },
    };
    more_option.click().await?;
    wait(2, 5);
    const IN_CONNECTION_POOL: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Remove your connection']";
    let in_connection_pool = browser.find(By::Css(IN_CONNECTION_POOL)).await;
    if in_connection_pool.is_ok() {
        let screenshot = browser.screenshot_as_png().await?;
        browser.quit().await?;
        send_screenshot(
            screenshot,
            &browser_info.user_id,
            "Connection accepted; withdrawal not possible",
            "Withdraw connection",
        )
        .await?;
        return Ok("Connection accepted; withdrawal not possible".to_string());
    }

    const PENDING_ON_THE_PAGE: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--secondary.ember-view.pvs-profile-actions__action[aria-label*='Pending']";
    const PENDING_DROPDOWN: &str = "div.artdeco-dropdown__item.artdeco-dropdown__item--is-dropdown.ember-view.full-width.display-flex.align-items-center[aria-label*='Pending']";

    let pending_on_the_page = browser.find(By::Css(PENDING_ON_THE_PAGE)).await;
    let pending_dropdown = browser.find(By::Css(PENDING_DROPDOWN)).await;
    match pending_on_the_page {
        Ok(button) => {
            button.click().await?;
        }
        Err(_) => match pending_dropdown {
            Ok(button) => {
                button.click().await?;
            }
            Err(_) => {
                let screenshot = browser.screenshot_as_png().await?;
                browser.quit().await?;
                send_screenshot(
                    screenshot,
                    &browser_info.user_id,
                    "Pending button is missing",
                    "Withdraw connection",
                )
                .await?;
                return Err(CustomError::ButtonNotFound(
                    "Pending button is missing".to_string(),
                ));
            }
        },
    };
    wait(1, 3);
    const WITHDRAW_BUTTON_POPUP: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--primary.ember-view.artdeco-modal__confirm-dialog-btn";
    //const SUCCESFULL_WITHDRAW_POPUP: &str =  "div.artdeco-toast-item.artdeco-toast-item--visible.ember-view";

    let withdraw_button_popup = browser.find(By::Css(WITHDRAW_BUTTON_POPUP)).await;
    match withdraw_button_popup {
        Ok(button) => {
            button.click().await?;
        }
        Err(_) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Withdraw popup button is missing",
                "Withdraw connection",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Withdraw popup button is missing".to_string(),
            ));
        }
    }

    wait(4, 6);
    const VERIFY_PROFILE: &str = "div.artdeco-modal.artdeco-modal--layer-default";
    const NOT_NOW_BUTTON: &str =
        "button.artdeco-button.artdeco-button--2.artdeco-button--secondary.ember-view.mr1";
    let verify_profile_popup = if let Ok(popup) = browser.find(By::Css(VERIFY_PROFILE)).await {
        Some(popup)
    } else {
        None
    };
    if verify_profile_popup.is_some() {
        let not_now_button = browser.find(By::Css(NOT_NOW_BUTTON)).await;
        match not_now_button {
            Ok(button) => {
                button.click().await?;
            }
            Err(_) => {
                let screenshot = browser.screenshot_as_png().await?;
                browser.quit().await?;
                send_screenshot(
                    screenshot,
                    &browser_info.user_id,
                    "Not now button is missing",
                    "Withdraw connection",
                )
                .await?;
                return Err(CustomError::ButtonNotFound(
                    "Not now button is missing".to_string(),
                ));
            }
        }
    }

    wait(2, 4);
    /*
    let succesfull_withdrow_popup = browser.find(By::Css(SUCCESFULL_WITHDRAW_POPUP)).await;
    match succesfull_withdrow_popup {
        Ok(_) => {
            browser.quit().await?;
            return Ok("Connection was withdrawn".to_string());
        }
        Err(_) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                "Succesfull popup is missing",
                "Withdraw connection",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Succesfull popup is missing".to_string(),
            ));
        }
    }
    */
    Ok("Connection was withdrawn".to_string())
}
