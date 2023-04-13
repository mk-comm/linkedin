use playwright::Playwright;
//use std::path::Path;


use playwright::api::{Cookie, ProxySettings, Viewport};
use std::{collections::HashMap};

use crate::structs::browser::BrowserConfig;
use crate::structs::user::User;
use crate::structs::entry::Entry;

use super::wait::wait;
pub async fn start_browser(entry: Entry) -> Result<BrowserConfig, playwright::Error> {
    
    //path to  local browser

    //let path = Path::new("/opt/homebrew/bin/chromium");



    let mut user = User::new(entry.user_agent, entry.session_cookie, entry.user_id);

    if user.user_agent.is_empty() {
        user.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36".to_string()
    } // use default user agent if not provided

    let proxy = ProxySettings {
        server: entry.ip,
        username: Some(entry.username),
        password: Some(entry.password),
        bypass: None,
    };

    let playwright = Playwright::initialize().await?;

    playwright.prepare()?; // Install browsers uncomment on production

    let chromium = playwright.chromium();

    let browser = chromium
        .launcher()
        .proxy(proxy)
        .headless(true)
        //.executable(path)
        .launch()
        .await?;
    
    let viewport = Viewport {
        width: 1920,
        height: 1080,
    };

    let context = browser.context_builder().viewport(Some(viewport.clone())).screen(viewport.clone()).build().await?;


    let mut headers = HashMap::new();

    headers.insert("User-Agent".to_string(), user.user_agent);

    context.set_extra_http_headers(headers).await?;

    //it appears only if you visit the target url, otherwise cookie won't show
    let cookie = Cookie::with_url(
        "li_at",
        user.session_cookie.as_str(),
        "https://.www.linkedin.com",
    );
    context.add_cookies(&[cookie]).await?;

    //testing proxy by visiting google.com

    let page_proxy = context.new_page().await?;

    let build_proxy = page_proxy
        .goto_builder("https://www.google.com/")
        .goto()
        .await;

    wait(1, 4);

    match &build_proxy {
        Ok(_) => print!("Proxy is working"),
        Err(_) => {
            wait(1, 3);
            browser.close().await?;
            return Err(playwright::Error::Timeout)
        },
    } // if error when proxy is not working


    let page = context.new_page().await?;

    let build = page
        .goto_builder("https://www.linkedin.com/feed/")
        .goto()
        .await?;

    wait(1, 4);
    
    let search_input = page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;
    match &search_input {
        Some(_) => print!("ok"),
        None => {
            wait(1, 3);
            browser.close().await?;
            return Err(playwright::Error::ReceiverClosed) // if error when session cookie expired
        },
    }
    
// if error when proxy is not working

    let browser_config = BrowserConfig {
        proxy: None,
        playwright: playwright,
        browser_type: chromium,
        browser: browser,
        context: context,
        page: page,
        build: build.unwrap(),
    };

    return Ok(browser_config);
}
