use crate::actions::start_browser::send_screenshot;
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
    let user_id = entry.user_id.clone();
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
    let ai_search = entry.aisearch.as_str();
    let _ = send_search_status("Starting the browser", ai_search).await;
    let browser = start_browser(browser_info).await?;

    let _ = send_search_status("Connected to Linkedin", ai_search).await;
    let _ = send_search_status("Opening the search page", ai_search).await;
    browser.page.goto_builder(&entry.url).goto().await?;
    wait(12, 15);

    let _ = send_search_status("Page opened", ai_search).await;
    let search_container = browser
        .page
        .query_selector("div.hp-core-temp.profile-list.profile-list-container")
        .await?;

    if search_container.is_none() {
        error!("search container not found");
        let screenshot = browser.page.screenshot_builder().screenshot().await?;
        send_screenshot(
            screenshot,
            &user_id,
            "Search list of candidates not found/Recruiter search",
            "scrap recruiter search",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Search list of candidates not found/Recruiter search".to_string(),
        ));
    }

    let _ = send_search_status("Counting candidates", ai_search).await;
    const CANDIDATE_NUMBER: &str =
        "span[class='profile-list__header-info-text t-14 profile-list__header-info-text--reflow']";
    let number_candidates = browser.page.query_selector(CANDIDATE_NUMBER).await?;
    let mut total_candidates: i32 = 0;
    match number_candidates {
        Some(number) => {
            let text = number.text_content().await?;
            let text = match text {
                Some(text) => text,
                None => String::from("1001"),
            };
            println!("{:?}", text);
            let result = if text.contains('M') || text.contains('K') {
                1000
            } else {
                let numeric_text: String = text.chars().filter(|c| c.is_digit(10)).collect();
                numeric_text.trim().parse::<i32>().unwrap_or(1002)
            };
            total_candidates = result;
            send_search_number(result, &entry.aisearch).await?
        }
        None => send_search_number(1003, &entry.aisearch).await?,
    };
    //let search_container = search_container.unwrap();

    /*
        let pages_count = count_pages(search_container.inner_html().await?);
        if pages_count.is_err() {
            error!("pages count not found");
            return Err(CustomError::ButtonNotFound(
                "Pages count not found/Scrap Recruiter Search".to_string(),
            ));
        }
        let pages_count = pages_count.unwrap();
        println!("pages count: {}", pages_count);
    */
    let total_candidates = format!("Total candidates: {}", total_candidates);
    let _ = send_search_status(total_candidates.as_str(), ai_search).await;
    let mut pages_number = 0;
    let mut pages_left = true;
    while pages_left {
        pages_number += 1;
        let page_scraped = format!("Started scraping page {}", pages_number);
        let _ = send_search_status(page_scraped.as_str(), ai_search).await;
        let mut url_list: Vec<String> = Vec::new();
        let search_container_inside = browser
            .page
            .query_selector("div.hp-core-temp.profile-list.profile-list-container")
            .await?;

        if search_container_inside.is_none() {
            error!("search container not found");
            let screenshot = browser.page.screenshot_builder().screenshot().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Search container not found/Scrap Recruiter Search",
                "scrap recruter search",
            )
            .await?;

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
        let result = send_urls(
            url_list,
            &entry.result_url,
            &entry.aisearch,
            &entry.url_list_id,
        )
        .await;

        match result {
            Ok(_) => {
                let _ = send_search_status(
                    format!(
                        "URL batch sent to bubble successfully, page: {}",
                        pages_number
                    )
                    .as_str(),
                    ai_search,
                )
                .await;
            }
            Err(_) => {
                let _ = send_search_status(
                    format!("Error sending URL batch to bubble, page: {}", pages_number).as_str(),
                    ai_search,
                )
                .await;
            }
        };

        let _ = send_search_status("Opening next page", ai_search).await;
        const NEXT_ICON: &str = ".mini-pagination__quick-link[rel='next']";

        const NEXT_BUTTON: &str = "a[class='pagination__quick-link pagination__quick-link--next']";
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

                    let _ = send_search_status("This was the last page", ai_search).await;
                    pages_left = false;
                }
                println!("next page not found");
                //break;
            }
        }
    }

    wait(5, 12);

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?; //close browser
    let _ = send_search_status("Search scraped successfully", ai_search).await;
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
/*
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
    url_list_id: &str,
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
        Ok(res) => {
            info!(
                "Send_urls/scrap_recruiter_search/Ok, {} was done, response {:?}",
                ai_search,
                res.status()
            );
            println!("API RESPONSE OK {:?}", res);
        }
        Err(error) => {
            error!(error = ?error, "Send_urls/scrap_recruiter_search/Error {} returned error {}", ai_search, error);
            println!("API RESPONSE {:?}", error);

            return Err(error);
        }
    }

    Ok(())
}
*/

/// Sends a list of URLs as JSON to a specified target URL with retries on failure.
///
/// # Arguments
/// * `urls` - A vector of URLs to send.
/// * `target_url` - The endpoint where the URLs will be sent.
/// * `ai_search` - A descriptive string associated with the search.
/// * `url_list_id` - Identifier for the list of URLs.
/// * `retry_delay` - Delay between retries in seconds.
///
/// # Returns
/// * `Result<(), reqwest::Error>` - Result of the POST request.
async fn send_urls(
    urls: Vec<String>,
    target_url: &str,
    ai_search: &str,
    url_list_id: &str,
) -> Result<(), CustomError> {
    let max_retries = 5;
    let client = reqwest::Client::new();

    for batch in urls.chunks(10) {
        let urls_json = json!({
            "urls": batch,
            "ai_search": ai_search,
            "url_list_id": url_list_id
        });

        let mut retries = 0;
        loop {
            let response = client.post(target_url).json(&urls_json).send().await;
            match response {
                Ok(res) => {
                    if res.status() == 200 {
                        info!(
                            "Send_urls/scrap_recruiter_search/Ok: {}, status: {}/URL {}",
                            ai_search,
                            res.status(),
                            target_url
                        );
                        break; // Proceed to the next batch
                    } else {
                        if retries < max_retries {
                            retries += 1;
                            wait(1, 1); // Wait 1 second before retrying
                            continue;
                        } else {
                            error!(
                                "Send_urls/scrap_recruiter_search/Error {}: status {}/URL: {}",
                                ai_search,
                                res.status(),
                                target_url
                            );
                            return Err(CustomError::ButtonNotFound(
                                "Send url status is not 200, Status/Scrap Recruiter Search"
                                    .to_string(),
                            ));
                        }
                    }
                }
                Err(error) => {
                    if retries < max_retries {
                        retries += 1;
                        wait(1, 1);
                        continue;
                    } else {
                        error!(error = ?error, "Send_urls/scrap_recruiter_search/Error {} returned error {}/URL: {}", ai_search, error, target_url);

                        return Err(CustomError::ButtonNotFound(
                            "Scrap recruiter search send url, Error".to_string(),
                        ));
                    }
                }
            }
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
    //println!("{:?}", response.text().await?);

    Ok(())
}
