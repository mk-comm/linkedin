use crate::actions::init_browser::{init_browser,send_screenshot};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapSearchRecruiter;
use crate::structs::error::CustomError;
use reqwest;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use serde_json::json;
use thirtyfour::By;
use thirtyfour::WebDriver;
use tracing::{error, info};

pub async fn scrap_recruiter_search(entry: EntryScrapSearchRecruiter) -> Result<(), CustomError> {
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
    let ai_search = entry.aisearch.as_str();
    let _ = send_search_status("Starting the browser", ai_search).await;
    let browser = init_browser(&browser_info).await?;
    let _ = send_search_status("Connected to Linkedin", ai_search).await;
    let _ = send_search_status("Opening the search page", ai_search).await;
    browser.goto(&entry.url).await?;
    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            wait(1, 3);
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;

            send_screenshot(
                screenshot,
                &user_id,
                "Recruiter Session cookie expired",
                "Start Browser",
                "Scrap Recruiter Search"
            )
            .await?;
            return Err(CustomError::RecruiterSessionCookieExpired);
        }

        println!("checking if cookie is valid{}", cookie_second_try);
    }
    wait(60, 65);
    let url_list_id = entry.url_list_id.to_string();
    let _ = send_search_status("Page opened", ai_search).await;

    const SEARCH_CONTAINER: &str = "div.hp-core-temp.profile-list.profile-list-container";
    let search_container = browser.find(By::Css(SEARCH_CONTAINER)).await;
    if search_container.is_err() {
        error!("search container not found");

        let screenshot = browser.screenshot_as_png().await?;
        send_screenshot(
            screenshot,
            &user_id,
            "Search list of candidates not found/Recruiter search",
            "scrap recruiter search",
            "Scrap Recruiter Search"
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Search list of candidates not found/Recruiter search".to_string(),
        ));
    }

    let _ = send_search_status("Counting candidates", ai_search).await;
    const CANDIDATE_NUMBER: &str =
        "span[class='profile-list__header-info-text t-14 profile-list__header-info-text--reflow']";
    let number_candidates = browser.find(By::Css(CANDIDATE_NUMBER)).await;
    let mut total_candidates: i32 = 0;
    match number_candidates {
        Ok(number) => {
            let text = number.text().await;
            let text = match text {
                Ok(text) => text,
                Err(_) => String::from("1001"),
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
        Err(_) => send_search_number(1003, &entry.aisearch).await?,
    };
    let total_candidates = format!("Total candidates: {}", total_candidates);
    let _ = send_search_status(total_candidates.as_str(), ai_search).await;
    let mut pages_number = 0;
    let mut pages_left = true;
    let mut number = entry.urls_scraped;
    let zoom_script = "document.body.style.zoom = '10.0%';";
    browser.execute(&zoom_script, vec![]).await?;
    while pages_left {
        pages_number += 1;
        let page_scraped = format!("Started scraping page {}", pages_number);
        let _ = send_search_status(page_scraped.as_str(), ai_search).await;
        let mut url_list: Vec<String> = Vec::new();
        const SEARCH_CONTAINER: &str = "div.hp-core-temp.profile-list.profile-list-container";

        let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
        if search_container_inside.is_err() {
            error!("search container not found");
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Search container not found/Scrap Recruiter Search",
                "scrap recruter search",
                "Scrap Recruiter Search"
            )
            .await?;

            return Err(CustomError::ButtonNotFound(
                "Search container not found/Scrap Recruiter Search".to_string(),
            ));
        }
        let search_container_inside = search_container_inside.unwrap();
        scroll(&browser).await?;
        wait(5, 7);
         let html = search_container_inside.inner_html().await?;
        
        
        let scrap = scrap(
            search_container_inside.inner_html().await?.as_str(),
            &mut url_list,
        );
                match scrap {
            Ok(_) => (),
            Err(error) => {
let screenshot = browser.screenshot_as_png().await?;
        send_screenshot(
            screenshot,
            &user_id,
            "Error scraping",
            "scrap recruiter search",
            "Scrap Recruiter Search"

        )
        .await?;
                return Err(error)},
        };
        
        let screenshot = browser.screenshot_as_png().await?;
        send_screenshot(
            screenshot,
            &user_id,
            "URL SENT",
            "Scrap recruiter search",
            "Scrap Recruiter Search"

        )
        .await?;
        if url_list.len() < 25 {
        let mut file = File::create(format!("ai_search{} page{}.txt",ai_search,pages_number)).unwrap();
        file.write_all(html.as_bytes()).unwrap();
        };
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
        number += 25;
        update_url_list(url_list_id.as_str(), number).await?;
        let _ = send_search_status("Opening next page", ai_search).await;
        const NEXT_ICON: &str = ".mini-pagination__quick-link[rel='next']";

        const NEXT_BUTTON: &str = "a[class='pagination__quick-link pagination__quick-link--next']";
        let next_page = browser.find(By::Css(NEXT_BUTTON)).await;
        match next_page {
            Ok(next_page) => {
                next_page.click().await?;
                wait(40, 45);
                wait(3, 5);
            }
            Err(_) => {
                let next_page = browser.find(By::Css(NEXT_ICON)).await;

                if next_page.is_ok() {
                    next_page.unwrap().click().await?;
                    wait(60, 65);
                } else {
                    println!("next page is empty");
                    let _ = send_search_status("This was the last page", ai_search).await;
                    pages_left = false;
                }
                
            }
        }
    }

    wait(5, 12);

    browser.quit().await?;
    let _ = send_search_status("Search scraped successfully", ai_search).await;
    Ok(())
}

pub async fn check_recruiter_cookie(page: &WebDriver) -> Result<bool, CustomError> {
    const SING_IN_TEXT_HEADER:&str = "h1.header__content__heading";
    let sign_in_text_header = page
        .find(By::Css(
            SING_IN_TEXT_HEADER
        ))
        .await;
     if sign_in_text_header.is_ok() {
if sign_in_text_header.unwrap().text().await?.trim() == "Sign in to LinkedIn Talent Solutions" {
    return Ok(false)
} else {
    Ok(true)
}
} else {
    Ok(true)
}
     
  


}

async fn scroll(page: &WebDriver) -> Result<(), CustomError> {
    let mut x = 0;

    while x < 25 {
        move_scroll(page).await?;
        x += 1;
    }

    Ok(())
}

async fn move_scroll(page: &WebDriver) -> Result<(), CustomError> {
    let scroll_code = r#"
        let scrollDistance = 365;
        window.scrollBy(0, scrollDistance);
    "#;
    page.execute(scroll_code, vec![]).await?;

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
