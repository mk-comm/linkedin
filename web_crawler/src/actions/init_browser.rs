use crate::actions::browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::error::CustomError;
use browser::chrome;
use serde_json::json;
use thirtyfour::cookie::SameSite;
use thirtyfour::{By, Cookie, WebDriver};
use tracing::{error, info};

pub async fn init_browser(browser_info: &BrowserInit) -> Result<WebDriver, CustomError> {
    let user_id = browser_info.user_id.clone();
    let mut browser = chrome(browser_info.ip.clone(), browser_info.user_agent.clone())
        .await
        .unwrap();

    add_cookie(
        browser_info.session_cookie.clone(),
        browser_info
            .recruiter_session_cookie
            .clone()
            .unwrap_or("".to_string()),
        &mut browser,
    )
    .await?;
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
                &user_id,
                "Session cookie expired",
                "Start Browser",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }

        println!("checking if cookie is valid{}", cookie_second_try);
    }
    Ok(browser)
}

async fn add_cookie(
    session_cookie: String,
    recruiter_cookie: String,
    driver: &mut WebDriver,
) -> Result<(), CustomError> {
    driver.goto("https://www.linkedin.com/login").await?;
    wait(1, 2);

    let mut session_cookie = Cookie::new("li_at", session_cookie);
    session_cookie.set_domain(".www.linkedin.com");
    session_cookie.set_path("/");
    session_cookie.set_same_site(Some(SameSite::Lax));

    let mut recruiter_cookie = Cookie::new("li_a", recruiter_cookie);
    recruiter_cookie.set_domain(".www.linkedin.com");
    recruiter_cookie.set_path("/");
    recruiter_cookie.set_same_site(Some(SameSite::Lax));
    driver.add_cookie(recruiter_cookie).await?;
    driver.add_cookie(session_cookie).await?;
    driver
        .goto("https://www.linkedin.com/mypreferences/d/categories/account")
        .await?;
    wait(1, 2);
    Ok(())
}

pub async fn session_cookie_is_valid(page: &WebDriver) -> Result<bool, CustomError> {
    let email_input = page
        .find(By::Css(
            "input[data-tracking-control-name='seo-authwall-base_join-form-email-or-phone']",
        ))
        .await;
    let email_input_another = page
        .find(By::Css("div[class='form__input--floating mt-24']"))
        .await;
    let sing_in_button = page
        .find(By::Css(
            "button[class='sign-in-modal__outlet-btn cursor-pointer btn-md btn-primary']",
        ))
        .await;
    let sing_in_button_main_screen = page
        .find(By::Css(
            "a[class='nav__button-secondary btn-secondary-emphasis btn-md']",
        ))
        .await;
    let sing_in_button_main_screen_another = page
        .find(By::Css(
            "a[class='nav__button-secondary btn-secondary-emphasis btn-sm ml-3']",
        ))
        .await;

    if email_input.is_ok() {
        Ok(false)
    } else {
        if sing_in_button.is_ok() {
            Ok(false)
        } else {
            if sing_in_button_main_screen.is_ok() {
                Ok(false)
            } else {
                if email_input_another.is_ok() {
                    Ok(false)
                } else {
                    if sing_in_button_main_screen_another.is_ok() {
                        Ok(false)
                    } else {
                        Ok(true)
                    }
                }
            }
        }
    }
}

pub async fn send_screenshot(
    screenshot: Vec<u8>,
    api_key: &str,
    variant: &str,
    message_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let screenshot = base64::encode(&screenshot);
    let send_json = json!({
        "screenshot": screenshot,
        "api_key":  api_key,
        "variant": variant,
        "message_id": message_id,
    });
    const TARGET_URL: &str = "
       https://tribexyz.bubbleapps.io/version-test/api/1.1/wf/sequence_status_screenshot/";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(TARGET_URL).json(&send_json).send().await;
    match response {
        Ok(_) => info!("{}/Ok, {} was done", variant, api_key),
        Err(error) => {
            error!(error = ?error, "{}/Error {} returned error {}", variant, api_key, error);
        }
    }

    Ok(())
}
