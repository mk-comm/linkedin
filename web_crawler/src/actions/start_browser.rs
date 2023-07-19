use playwright::Playwright;
use std::path::Path;

use playwright::api::{Cookie, ProxySettings, Viewport};
use std::collections::HashMap;
use crate::structs::error::CustomError;
use crate::structs::browser::{BrowserConfig, BrowserInit};
use crate::structs::user::User;
use tracing::{info};
use super::wait::wait;

pub async fn start_browser(browserinfo: BrowserInit) -> Result<BrowserConfig, CustomError> {
    info!("Starting browser");
    //path to  local browser
    
    let path = Path::new("/opt/homebrew/bin/chromium");

    let mut user = User::new(browserinfo.user_agent, browserinfo.session_cookie, browserinfo.user_id);

    if user.user_agent.is_empty() {
        user.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36".to_string()
    } // use default user agent if not provided

    let proxy = ProxySettings {
        server: browserinfo.ip,
        username: Some(browserinfo.username),
        password: Some(browserinfo.password),
        bypass: None,
    };

    let playwright = Playwright::initialize().await?;

    //let _ = playwright.prepare(); // Install browsers uncomment on production

    let chromium = playwright.chromium();

    let browser = chromium
        .launcher()
        .proxy(proxy)
        .headless(false)
        
        .executable(path)
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
    
    let cookie_recruiter = match browserinfo.recruiter_session_cookie {
        Some(cookie) => Cookie::with_url(
            "li_a",
            cookie.as_str(),
            "https://www.linkedin.com",
        ),
        None => {
            Cookie::with_url(
                "li_a",
                "79cb93e0632ed9960f88cbdd1e3361d4a9e64fbe",
                "https://www.linkedin.com",
            )
        }
    };


 // if recruiter cookie is not provided
        
    context.add_cookies(&[cookie,cookie_recruiter]).await?;
    
    //testing proxy by visiting google.com
    wait(3, 7);
    let page_proxy = context.new_page().await?;
 
    let _build_proxy = page_proxy
        .goto_builder("https://www.google.com/")
        .goto()
        .await;

    wait(1, 4);

    let google_search = page_proxy
    .query_selector("div.RNNXgb")
    .await?;

    match google_search {
        Some(_) => println!("proxy is working"),
        None => {
            println!("proxy is not working");
            wait(1, 3);
            page_proxy.close(Some(false)).await?;
            browser.close().await?;

            return Err(CustomError::ProxyNotWorking);
        }
    } // if error when proxy is not working
    
    page_proxy.close(Some(false)).await?;
    
    let page = context.new_page().await?;
    wait(1, 5);
     // close proxy page˚

    let build = page.goto_builder("https://www.linkedin.com/feed/");
    wait(1, 3);
    
    let mut go_to: Result<Option<playwright::api::Response>, std::sync::Arc<playwright::Error>> = build.goto().await;
    let mut x = 0;
    if go_to.is_err() {
        
        while x <= 3 {
            wait(3, 6);
            let build: Result<Option<playwright::api::Response>, std::sync::Arc<playwright::Error>> = page.goto_builder("https://www.linkedin.com/feed/")
            .goto().await;
            if build.is_ok() {
                go_to = build;
                break;
            } else if build.is_err() && x == 3 {
                wait(1, 3);
                page.close(Some(false)).await?;
                browser.close().await?;
                return Err(CustomError::ButtonNotFound("Feed is not loading".to_string())); // if error means page is not loading
            }
            x += 1;
            println!("retrying to load page")
        }
        wait(1, 3);
    } else {
        wait(1, 3);
    }

    //page.evaluate(r#"window.stop()"#, ()).await?;
  

    wait(7, 14);

    let profile = page
        .query_selector("div.feed-identity-module__actor-meta.break-words")
        .await?;
    
    match &profile {
        Some(_) => (),
        None => {
            wait(1, 3);
            page.close(Some(false)).await?;
            browser.close().await?;
            return Err(CustomError::SessionCookieExpired); // if error when session cookie expired
        }
    }

    // if error when proxy is not working

    let browser_config = BrowserConfig {
        proxy: None,
        playwright,
        browser_type: chromium,
        browser,
        context,
        page,
        build: go_to.unwrap().unwrap(),
    };


    return Ok(browser_config);
}
