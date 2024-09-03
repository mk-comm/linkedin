use crate::structs::browser::{BrowserConfigNew, BrowserInit};
use crate::structs::error::CustomError;
use crate::structs::user::User;
use base64;
use playwright::Playwright;
use tokio::time::{timeout, Duration};

use playwright::api::{Cookie, Page, ProxySettings, Viewport};
use reqwest::{Client, Error, Proxy};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

pub async fn start_browser(browserinfo: BrowserInit) -> Result<BrowserConfigNew, CustomError> {
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
        .headless(true)
        .chromium_sandbox(false)
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
    if let Some(bcookie_value) = browserinfo.bcookie {
        let bcookie = Cookie::with_url(
            "bcookie",
            bcookie_value.as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[bcookie]).await?;
    }

    if let Some(bscookie_value) = browserinfo.bscookie {
        let bscookie = Cookie::with_url(
            "bscookie",
            bscookie_value.as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[bscookie]).await?;
    }

    if let Some(fcookie_value) = browserinfo.fcookie {
        let fcookie = Cookie::with_url(
            "fcookie",
            fcookie_value.as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[fcookie]).await?;
    }

    if let Some(fidcookie_value) = browserinfo.fidcookie {
        let fid_cookie = Cookie::with_url(
            "fid_cookie",
            fidcookie_value.as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[fid_cookie]).await?;
    }

    if let Some(jsessionid_value) = browserinfo.jsessionid {
        let jsessionid = Cookie::with_url(
            "JSESSIONID",
            jsessionid_value.as_str(),
            "https://www.linkedin.com",
        );
        context.add_cookies(&[jsessionid]).await?;
    }

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

    let browser_config = BrowserConfigNew {
        proxy: None,
        playwright,
        browser_type: chromium,
        browser,
        context,
    };

    Ok(browser_config)
}

pub async fn session_cookie_is_valid(page: &Page) -> Result<bool, CustomError> {
    let email_input = page
        .query_selector(
            "input[data-tracking-control-name='seo-authwall-base_join-form-email-or-phone']",
        )
        .await?;
    let sing_in_button = page
        .query_selector(
            "button[class='sign-in-modal__outlet-btn cursor-pointer btn-md btn-primary']",
        )
        .await?;
    let sing_in_button_main_screen = page
        .query_selector("a[class='sign-in-modal__outlet-btn cursor-pointer btn-md btn-primary']")
        .await?;

    println!("email_input{:?}", email_input);
    println!("signin_input{:?}", sing_in_button);
    if email_input.is_some() {
        Ok(false)
    } else {
        if sing_in_button.is_some() {
            Ok(false)
        } else {
            if sing_in_button_main_screen.is_some() {
                Ok(false)
            } else {
                Ok(true)
            }
        }
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

        match timeout(Duration::from_secs(5), request).await {
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
