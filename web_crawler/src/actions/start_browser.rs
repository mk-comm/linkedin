use playwright::Playwright;
use tokio::time::{timeout, Duration};
//use std::path::Path;
use super::wait::wait;
use crate::structs::browser::{BrowserConfig, BrowserInit};
use crate::structs::error::CustomError;
use crate::structs::user::User;
use base64;

use playwright::api::{Cookie, Page, ProxySettings, Viewport};
use reqwest::{Client, Error, Proxy};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

pub async fn start_browser(browserinfo: BrowserInit) -> Result<BrowserConfig, CustomError> {
    let proxy_check = format!(
        "{}:{}@{}",
        browserinfo.username, browserinfo.password, browserinfo.ip
    );
    let proxy_validation = valid_proxy(proxy_check.as_str()).await?;
    if proxy_validation == false {
        return Err(CustomError::ProxyNotWorking);
    };

    info!("Starting browser");
    //path to  local browser

    //   let path = Path::new("/opt/homebrew/bin/chromium");
    let proxy = ProxySettings {
        server: browserinfo.ip,
        username: Some(browserinfo.username),
        password: Some(browserinfo.password),
        bypass: None,
    };
    //let path = Path::new("./chrome-linux/chrome");
    let mut user = User::new(
        browserinfo.user_agent,
        browserinfo.session_cookie,
        browserinfo.user_id,
    );

    if user.user_agent.is_empty() {
        user.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36".to_string()
    } // use default user agent if not provided

    let playwright = Playwright::initialize().await?;

    let _ = playwright.prepare(); // Install browsers uncomment on production

    let chromium = playwright.chromium();

    let browser = chromium
        .launcher()
        .proxy(proxy.clone())
        .headless(browserinfo.headless)
        //.executable(path)
        .launch()
        .await?;

    let viewport = Viewport {
        width: 1920,
        height: 1080,
    };

    let context = browser
        .context_builder()
        .viewport(Some(viewport.clone()))
        .screen(viewport.clone())
        .build()
        .await?;

    let mut headers = HashMap::new();

    headers.insert("User-Agent".to_string(), user.user_agent);

    context.set_extra_http_headers(headers).await?;

    //it appears only if you visit the target url, otherwise cookie won't show
    let cookie = Cookie::with_url(
        "li_at",
        user.session_cookie.as_str(),
        "https://.www.linkedin.com",
    );

    /*
    let bcookie = Cookie::with_url(
        "bcookie",
        "\"v=2&5f07fbd5-eb98-48ee-88dc-09423cf73043\"",
        "https://.www.linkedin.com",
    );
    let bscookie = Cookie::with_url(
        "bscookie",
        "\"v=1&2024082110083769541e04-71e8-4b50-8121-dd35c6c820d3AQGmQbaMve2h6qWSlWnCxlZ_Ei_vj_uS\"",
        "https://.www.linkedin.com",
    );
            let fcookie = Cookie::with_url(
                "fcookie",
                "\"v=1&2024082110083769541e04-71e8-4b50-8121-dd35c6c820d3AQGmQbaMve2h6qWSlWnCxlZ_Ei_vj_uS\
        ",
                "https://.www.linkedin.com",
            );let fid_cookie = Cookie::with_url(
            "fid_cookie",
            "\"ajax:8364603528610338839\"
    ",
            "https://.www.linkedin.com",
        );
    let JSESSIONID = Cookie::with_url(
        "JSESSIONID",
        "\"ajax:8364603528610338839\"",
        "https://.www.linkedin.com",
    );

        */

    // if recruiter cookie is not provided

    context.add_cookies(&[cookie]).await?;
    if browserinfo.recruiter_session_cookie.is_some() {
        let recruiter_cookie = Cookie::with_url(
            "li_a",
            browserinfo.recruiter_session_cookie.unwrap().as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[recruiter_cookie]).await?;
    }
    const INITIAL_PAGE: &str = "https://www.linkedin.com/mypreferences/d/categories/account";
    let page = context.new_page().await?;
    let build = page.goto_builder(INITIAL_PAGE);
    wait(1, 3);

    let mut go_to: Result<Option<playwright::api::Response>, std::sync::Arc<playwright::Error>> =
        build.goto().await;

    wait(7, 14);
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build: Result<
                Option<playwright::api::Response>,
                std::sync::Arc<playwright::Error>,
            > = page.goto_builder(INITIAL_PAGE).goto().await;
            if build.is_ok() {
                go_to = build;
                break;
            } else if build.is_err() && x == 3 {
                wait(1, 3);
                let screenshot = page.screenshot_builder().screenshot().await?;
                page.close(Some(false)).await?;
                browser.close().await?;
                send_screenshot(
                    screenshot,
                    &user.user_id,
                    "Feed is not loading",
                    "Start Browser",
                )
                .await?;
                return Err(CustomError::ButtonNotFound(
                    "Feed is not loading".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            //println!("retrying to load page")
        }
        wait(1, 3);
    } else {
        wait(1, 3);
    }

    wait(7, 14);
    let cookie = session_cookie_is_valid(&page).await?;
    if !cookie {
        page.reload_builder().reload().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&page).await?;
        if !cookie_second_try {
            wait(1, 3);
            let screenshot = page.screenshot_builder().screenshot().await?;
            page.close(Some(false)).await?;
            browser.close().await?;
            send_screenshot(
                screenshot,
                &user.user_id,
                "Session cookie expired",
                "Start Browser",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }

        println!("checking if cookie is valid{}", cookie_second_try);
    }
    println!("checking if cookie is valid{}", cookie);

    let browser_config = BrowserConfig {
        proxy: None,
        playwright,
        browser_type: chromium,
        browser,
        context,
        page,
        build: go_to.unwrap().unwrap(),
        headless: browserinfo.headless,
    };

    Ok(browser_config)
}

async fn session_cookie_is_valid(page: &Page) -> Result<bool, CustomError> {
    wait(1, 3);
    let email_input = page.query_selector("input[name=email-address]").await?;
    if email_input.is_some() {
        Ok(false)
    } else {
        Ok(true)
    }
}
async fn valid_proxy(proxy_url: &str) -> Result<bool, Error> {
    let proxy = Proxy::all(proxy_url)?;
    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(2))
        .build()?;

    // List of URLs to test against
    let test_urls = vec!["https://www.google.com", "https://www.example.com"];

    for test_url in test_urls {
        let request = client.get(test_url).send();

        match timeout(Duration::from_secs(2), request).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    println!("Proxy {} is working for {}", proxy_url, test_url);
                    return Ok(true);
                } else {
                    println!(
                        "Proxy {} returned a non-success status for {}: {}",
                        proxy_url,
                        test_url,
                        response.status()
                    );
                }
            }
            Ok(Err(err)) => {
                println!(
                    "Error sending request through proxy {} for {}: {}",
                    proxy_url, test_url, err
                );
            }
            Err(_) => {
                println!(
                    "Request to {} timed out through proxy {}",
                    test_url, proxy_url
                );
            }
        }
    }

    Ok(false)
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
        Ok(_) => info!(
            "Send_search_number/send_screenshot_expired_session_cookie/Ok, {} was done",
            api_key
        ),
        Err(error) => {
            error!(error = ?error, "Send_search_number/send_screenshot_expired_session_cookie/Error {} returned error {}", api_key, error);
        }
    }

    Ok(())
}
