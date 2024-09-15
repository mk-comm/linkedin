use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapSearchRegular;
use crate::structs::error::CustomError;
use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use thirtyfour::{By, WebDriver};
use tracing::{error, info};

pub async fn scrap_regular_search(entry: EntryScrapSearchRegular) -> Result<(), CustomError> {
    let url = entry.url;
    let ai_search = entry.aisearch;
    let url_list_id = entry.url_list_id;
    let result_url = entry.result_url;
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
    let result = start_scrap(
        &browser,
        &url,
        &ai_search,
        &url_list_id,
        &result_url,
    )
    .await;
    match result {
        Ok(text) => {
            browser.quit().await?;
            return Ok(text);
        }
        Err(error) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                error.to_string().as_str(),
                "Scrap regular search",
                "Scrap regular search",
            )
            .await?;
            return Err(error);
        }
    }
}

pub async fn start_scrap(
    browser: &WebDriver,
    url: &str,
    ai_search: &str,
      url_list_id: &str,
    result_url: &str,
) -> Result<(), CustomError> {
    let go_to = browser.goto(url).await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(url).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                return Err(CustomError::ButtonNotFound(
                    "Scrap regular search page is not loading".to_string(),
                ));
            }
            x += 1;
        }
        wait(1, 3);
    }
    wait(7, 10);
    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const CANDIDATE_NUMBER: &str = "h2.pb2.t-black--light.t-14";
    let number_candidates = browser.find(By::Css(CANDIDATE_NUMBER)).await;

    match number_candidates {
        Ok(number) => {
            let text = number.text().await;
            let text = match text {
                Ok(text) => {
                    println!("Number of candidates in the search {}", text);
                    text
                }
                Err(_) => String::from("1001"),
            };
            let result = if text.contains(',') || text.contains('K') {
                1000
            } else {
                let numeric_text: String = text.chars().filter(|c| c.is_digit(10)).collect();
                println!("numeric {}", numeric_text);
                numeric_text.trim().parse::<i32>().unwrap_or(1002)
            };
            send_search_number(result, ai_search).await?
        }
        Err(_) => send_search_number(1003, ai_search).await?,
    };
    let url_list_id = url_list_id.to_string();
    const SEARCH_CONTAINER: &str = "div.search-results-container";

    let search_container = browser.find(By::Css(SEARCH_CONTAINER)).await?;

    let pages_count = count_pages(search_container.inner_html().await?);
    println!("pages count: {}", pages_count);

    for i in 1..=pages_count {
        let mut url_list: Vec<String> = Vec::new();
        let page_number = format!("button[aria-label='Page {}']", i);
        let next_page = browser.find(By::Css(page_number.as_str())).await?;
        next_page.click().await?;
        wait(7, 10);
        let cookie = session_cookie_is_valid(&browser).await?;
        if !cookie {
            browser.refresh().await?;
            wait(7, 14);
            let cookie_second_try = session_cookie_is_valid(&browser).await?;
            if !cookie_second_try {
                return Err(CustomError::SessionCookieExpired);
            }
        }
        const CONTAINER: &str = "div.search-results-container";
        let container = browser.find(By::Css(CONTAINER)).await?;

        scrap(container.inner_html().await?.as_str(), &mut url_list);
        wait(1, 2);
        send_urls(url_list, result_url, ai_search, &url_list_id).await?;
        update_url_list(url_list_id.as_str(), i).await?;
        wait(3, 5);
    }

    wait(5, 12);

    let _ = send_search_status("Search scraped successfully", &ai_search).await;

    Ok(())
}
async fn update_url_list(url_list_id: &str, number: i32) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({ 
        "url_list_id": url_list_id,
        "number": number});
    let target_url = "https://overview.tribe.xyz/api/1.1/wf/tribe_update_url";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!(
            "Update_url_list/scrap_recruiter_search/Ok, {} was done",
            url_list_id
        ),
        Err(error) => {
            error!(error = ?error, "Update_url_list/scrap_recruiter_search/Error {} returned error {}", url_list_id, error);
        }
    }

    Ok(())
}
async fn send_search_status(status: &str, ai_search: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({ 
        "status": status,
        "ai_search": ai_search});
    let target_url = "https://overview.tribe.xyz/api/1.1/wf/search_logs";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!(
            "Send_search_status/scrap_recruiter_search/Ok, {} was done",
            ai_search
        ),
        Err(error) => {
            error!(error = ?error, "Send_search_status/scrap_recruiter_search/Error {} returned error {}", ai_search, error);
        }
    }

    Ok(())
}
fn scrap(html: &str, url_list: &mut Vec<String>) {
    let document = Html::parse_document(html);

    // Define a selector for the LinkedIn URLs
    let a_selector = Selector::parse("span.entity-result__title-text > a.app-aware-link").unwrap();

    // Extract LinkedIn URLs
    let linkedin_urls: Vec<String> = document
        .select(&a_selector)
        .filter_map(|el| el.value().attr("href"))
        .map(String::from)
        .collect();

    // Print the results
    for url in &linkedin_urls {
        url_list.push(url.to_string());
    }
}

fn count_pages(html: String) -> i32 {
    let document = Html::parse_document(html.as_str());

    // Selector for the last page button
    let last_page_selector =
        Selector::parse("li.artdeco-pagination__indicator--number:last-child button").unwrap();
    let last_page_elem = document.select(&last_page_selector).next().unwrap();
    let aria_label = last_page_elem.value().attr("aria-label").unwrap();
    let total_pages: i32 = aria_label
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap();

    total_pages
}

async fn send_urls(
    urls: Vec<String>,
    target_url: &str,
    ai_search: &str,
    url_list_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({ 
        "urls": urls,
        "ai_search": ai_search,
        "url_list_id": url_list_id     });

    let response = client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!("Send_urls/scrap_regular_search/Ok, {} was done", ai_search),
        Err(error) => {
            error!(error = ?error, "Send_urls/scrap_regular_search/Error {} returned error {}", ai_search, error);
        }
    }

    Ok(())
}
async fn send_search_number(number: i32, ai_search: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let send_json = json!({ 
        "number": number,
        "ai_search": ai_search});
    const TARGET_URL: &str = "
        https://overview.tribe.xyz/api/1.1/wf/tribe_search_number";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(TARGET_URL).json(&send_json).send().await;
    match response {
        Ok(_) => info!(
            "Send_search_number/scrap_recruiter_search/Ok, {} was done",
            ai_search
        ),
        Err(error) => {
            error!(error = ?error, "Send_search_number/scrap_recruiter_search/Error {} returned error {}", ai_search, error);
        }
    }

    Ok(())
}
