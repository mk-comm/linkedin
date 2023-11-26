use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::error::CustomError;
use crate::{actions::start_browser::start_browser, structs::entry::EntryScrapSearchRecruiter};

use playwright::api::Page;
use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use tracing::{error, info};

pub async fn scrap_recruiter_search(entry: EntryScrapSearchRecruiter) -> Result<(), CustomError> {
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

    browser.page.goto_builder(&entry.url).goto().await?;

    wait(12, 15);

    //let pagination = browser
    //.page
    //.query_selector("ol.pagination__list")
    //.await?
    //;

    //wait(10000, 20000);

    let search_container = browser
        .page
        .query_selector("div.hp-core-temp.profile-list.profile-list-container")
        .await?;

    if search_container.is_none() {
        error!("search container not found");
        return Err(CustomError::ButtonNotFound(
            "Search container not found/Scrap Recruiter Search".to_string(),
        ));
    }
    let search_container = search_container.unwrap();

    let pages_count = count_pages(search_container.inner_html().await?);

    if pages_count.is_err() {
        error!("pages count not found");
        return Err(CustomError::ButtonNotFound(
            "Pages count not found/Scrap Recruiter Search".to_string(),
        ));
    }
    let pages_count = pages_count.unwrap();
    println!("pages count: {}", pages_count);
    for i in 1..=pages_count {
        let mut url_list: Vec<String> = Vec::new();
        let search_container_inside = browser
            .page
            .query_selector("div.hp-core-temp.profile-list.profile-list-container")
            .await?;

        if search_container_inside.is_none() {
            error!("search container not found");
            return Err(CustomError::ButtonNotFound(
                "Search container not found/Scrap Recruiter Search".to_string(),
            ));
        }
        let search_container_inside = search_container_inside.unwrap();
        scroll(&browser.page).await?;
        wait(5, 7);
        let _scrap = scrap(
            search_container_inside.inner_html().await?.as_str(),
            &mut url_list,
        );
        send_urls(url_list, &entry.result_url, &entry.aisearch, &entry.url_list_id).await?;
        const NEXT_ICON: &str = ".mini-pagination__quick-link[rel='next']";

        const NEXT_BUTTON: &str =
            "a[class='pagination__quick-link pagination__quick-link--next']";
        let next_page = browser.page.query_selector(NEXT_BUTTON).await?;
        match next_page {
            Some(next_page) => {
                next_page.click_builder().click().await?;
                wait(10, 15);
                //pagination.scroll_into_view_if_needed(Some(1000.0)).await?;
                //println!("url list: {:?}", url_list);
                wait(3, 5);
            }
            None => {
                let next_page = browser.page.query_selector(NEXT_ICON).await?;
                if next_page.is_some() {
                    next_page.unwrap().click_builder().click().await?;
                    wait(10, 15);
                } else {
                    println!("next page is empty");
                }
                println!("next page not found");
                //break;
            }
        }
    }

    wait(5, 12);

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?; // close browser
    Ok(())
}

async fn scroll(page: &Page) -> Result<(), CustomError> {
    let mut x = 0;

    while x < 25 {
        move_scroll(page).await?;
        x += 1;
    }

    Ok(())
}

async fn move_scroll(page: &Page) -> Result<(), CustomError> {
    let scroll_code = r#"
    function() {
        let totalHeight = document.body.scrollHeight;
        let scrollDistance = 365;
        window.scrollBy(0, scrollDistance);
    }
    "#;

    page.evaluate(scroll_code, ()).await?;

    wait(1, 2);
    Ok(())
}

fn scrap(html: &str, url_list: &mut Vec<String>) -> Result<(), CustomError> {
    let document = Html::parse_document(html);

    // Define a selector for the LinkedIn URLs
    let a_selector = Selector::parse("a[data-test-link-to-profile-link='true']");

    if a_selector.is_err() {
        return Err(CustomError::ButtonNotFound(
            "a_selector is error/Scrap Recruiter Search".to_string(),
        ));
    }
    let a_selector = a_selector.unwrap();

    // Extract LinkedIn URLs
    let linkedin_urls: Vec<String> = document
        .select(&a_selector)
        .filter_map(|el| el.value().attr("href"))
        .map(String::from)
        .collect();

    // Print the results
    for url in &linkedin_urls {
        let urn = extract_part_from_url(url);
        let complete_url = format!("https://www.linkedin.com/in/{}", urn.unwrap());
        url_list.push(complete_url.to_string());
    }
    Ok(())
}

fn extract_part_from_url(url: &str) -> Option<&str> {
    // Check if the URL contains '/talent/profile/' and then split by that
    let parts: Vec<&str> = url.split("/talent/profile/").collect();
    if parts.len() > 1 {
        // Now we split by the '?' character to get the desired portion
        let desired_parts: Vec<&str> = parts[1].split('?').collect();
        Some(desired_parts[0])
    } else {
        None
    }
}

fn count_pages(html: String) -> Result<i32, CustomError> {
    let document = Html::parse_document(&html);

    // Selector for the last pagination number
    let last_page_selector =
        Selector::parse("li.pagination__link:last-child span[aria-hidden='true']");

    if last_page_selector.is_err() {
        return Err(CustomError::ButtonNotFound(
            "last_page_selector is error/Scrap Recruiter Search".to_string(),
        ));
    }
    let last_page_selector = last_page_selector.unwrap();

    if let Some(last_page_elem) = document.select(&last_page_selector).next() {
        if let Some(text_content) = last_page_elem.text().next() {
            return Ok(text_content.trim().parse().unwrap_or(1));
        }
    }

    Ok(1) // Default to 1 if the element isn't found or doesn't contain a valid number
}

async fn send_urls(
    urls: Vec<String>,
    target_url: &str,
    ai_search: &str,
    url_list_id: &str
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({ 
        "urls": urls,
        "ai_search": ai_search,
    "url_list_id": url_list_id});

    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!(
            "Send_urls/scrap_recruiter_search/Ok, {} was done",
            ai_search
        ),
        Err(error) => {
            error!(error = ?error, "Send_urls/scrap_recruiter_search/Error {} returned error {}", ai_search, error);
        }
    }
    //println!("{:?}", response.text().await?);

    Ok(())
}
