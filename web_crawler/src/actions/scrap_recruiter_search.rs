use crate::actions::init_browser::{init_browser, send_screenshot};
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapSearchRecruiter;
use crate::structs::error::CustomError;
use reqwest;
use scraper::{Html, Selector};
use serde_json::json;
use thirtyfour::By;
use thirtyfour::WebDriver;
use tracing::{error, info};

pub async fn scrap_recruiter_search(entry: EntryScrapSearchRecruiter) -> Result<String, CustomError> {
    let user_id = entry.user_id.clone();
    let url = entry.url;
    let url_scraped = entry.urls_scraped;
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
    let ai_search = entry.aisearch.as_str();
    let _ = send_search_status("Starting the browser", ai_search).await;
    let browser = init_browser(&browser_info).await?;

    let result = scrap_recruiter(&browser, &user_id, &ai_search,&url, &url_list_id, url_scraped, &result_url).await;
        match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                text.as_str(),
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;
            return Ok(text);
        }
        Err(error) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                error.to_string().as_str(),
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;
            return Err(error);
        }
    }

}
pub async fn scrap_recruiter(browser: &WebDriver, user_id:&str, ai_search: &str, url:&str, url_list_id: &str, url_scraped: i32, result_url: &str) -> Result<String, CustomError> {
    let _ = send_search_status("Connected to Linkedin", ai_search).await;
    let _ = send_search_status("Opening the search page", ai_search).await;
    browser.goto(&url).await?;
    const SESSIONS_SCREEN:&str = "main[id=sessions-container]";
    let sessions_screen = browser.find(By::Css(SESSIONS_SCREEN)).await;
    if sessions_screen.is_ok() {
        const BUTTON_CONTINUE:&str = "button.artdeco-button.artdeco-button--primary";
        let button_continue = browser.find(By::Css(BUTTON_CONTINUE)).await;
        match button_continue  {
            Ok(button) => {button.click().await?;
            wait(10, 12);},
            Err(_) => return Err(CustomError::ButtonNotFound("Button continue not found on the multiple sessions screen".to_string())),
        }
    }

    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::RecruiterSessionCookieExpired);
        }

        println!("checking if cookie is valid{}", cookie_second_try);
    }

    wait(20, 35);
    let url_list_id = url_list_id;
    let _ = send_search_status("Page opened", ai_search).await;

    const SEARCH_CONTAINER: &str = "div.hp-core-temp.profile-list.profile-list-container";
    let search_container = browser.find(By::Css(SEARCH_CONTAINER)).await;
    if search_container.is_err() {
        browser.refresh().await?;
        wait(20, 35);
        let search_container = browser.find(By::Css(SEARCH_CONTAINER)).await;
        if search_container.is_err() {
            browser.refresh().await?;
            wait(20, 35);
            let search_container = browser.find(By::Css(SEARCH_CONTAINER)).await;
            if search_container.is_err() {
                error!("search container not found");

                return Err(CustomError::ButtonNotFound(
                    "Search list of candidates not found/Recruiter search".to_string(),
                ));
            }
        }
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
            send_search_number(result, &ai_search).await?
        }
        Err(_) => send_search_number(1003, &ai_search).await?,
    };
    let total_candidates = format!("Total candidates: {}", total_candidates);
    let _ = send_search_status(total_candidates.as_str(), ai_search).await;
    let mut pages_number = 1;
    let mut pages_left = true;
    let mut number = url_scraped;
    let zoom_script = "document.body.style.zoom = '70.0%';";
    browser.execute(&zoom_script, vec![]).await?;
    while pages_left {
        let mut url_list: Vec<String> = Vec::new();
        const SEARCH_CONTAINER: &str = "div.hp-core-temp.profile-list.profile-list-container";

        let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
        if search_container_inside.is_err() {
            browser.refresh().await?;
            wait(20, 35);
            let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
            if search_container_inside.is_err() {
                browser.refresh().await?;
                wait(20, 35);
                let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
                if search_container_inside.is_err() {
                    error!("search container not found");
                    return Err(CustomError::ButtonNotFound(
                        "Search list of candidates inside not found/Recruiter search".to_string(),
                    ));
                }
            }
        }
        scroll(&browser, ScrollDirection::Top).await?;
        scroll(&browser, ScrollDirection::Bottom).await?;
        let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
        let search_container_inside = search_container_inside.unwrap();
        wait(15, 17);
        const EMPTY_PROFILE: &str =
            "li.ember-view.profile-list__occlusion-area.profile-list__border-bottom";
        let empty_profile = search_container_inside.find(By::Css(EMPTY_PROFILE)).await;
        if empty_profile.is_ok() {
            info!("EMPTY PROFILE FOUND");
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "First Less than 25 urls",
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;

            wait(15, 21);
        scroll(&browser, ScrollDirection::Top).await?;
                        let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "First Less than 25 urls TOP",
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;

        wait(5,10);
        scroll(&browser, ScrollDirection::Bottom).await?;

        }
        let search_container_inside = browser.find(By::Css(SEARCH_CONTAINER)).await;
        let search_container_inside = search_container_inside.unwrap();
        let empty_profile = search_container_inside.find(By::Css(EMPTY_PROFILE)).await;
        if empty_profile.is_ok() {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Second Less than 25 urls",
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;

            wait(30, 31);
            scroll(&browser, ScrollDirection::Top).await?;
            wait(5, 10);
            scroll(&browser, ScrollDirection::Bottom).await?;
        }

        let scrap = scrap(
            search_container_inside.inner_html().await?.as_str(),
            &mut url_list,
        );
        match scrap {
            Ok(_) => (),
            Err(error) => {
                return Err(error);
            }
        };

        if url_list.len() < 25 {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Less than 25 urls",
                &ai_search,
                "Scrap Recruiter Search",
            )
            .await?;
        };
        let result = send_urls(
            url_list,
            &result_url,
            &ai_search,
            &url_list_id,
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
        update_url_list(url_list_id, number).await?;
        let _ = send_search_status("Opening next page", ai_search).await;
        const NEXT_ICON: &str = ".mini-pagination__quick-link[rel='next']";

        const NEXT_BUTTON: &str = "a[class='pagination__quick-link pagination__quick-link--next']";
        let next_page = browser.find(By::Css(NEXT_BUTTON)).await;
        match next_page {
            Ok(next_page) => {
                next_page.click().await?;
                wait(15, 25);
            }
            Err(_) => {
                let next_page = browser.find(By::Css(NEXT_ICON)).await;

                if next_page.is_ok() {
                    next_page.unwrap().click().await?;
                wait(15, 25);
                } else {
                    println!("next page is empty");
                    let _ = send_search_status("This was the last page", ai_search).await;
                    pages_left = false;
                }
            }
        }
        pages_number += 1;
        let page_scraped = format!("Started scraping page {}", pages_number);
        move_scroll_top(&browser).await?;
        let _ = send_search_status(page_scraped.as_str(), ai_search).await;
    }

    wait(5, 12);

    let _ = send_search_status("Search scraped successfully", ai_search).await;
    Ok("Scraping recruiter search was successful!".to_string())
}

pub async fn check_recruiter_cookie(page: &WebDriver) -> Result<bool, CustomError> {
    const SING_IN_TEXT_HEADER: &str = "h1.header__content__heading";
    let sign_in_text_header = page.find(By::Css(SING_IN_TEXT_HEADER)).await;
    if sign_in_text_header.is_ok() {
        if sign_in_text_header.unwrap().text().await?.trim()
            == "Sign in to LinkedIn Talent Solutions"
        {
            return Ok(false);
        } else {
            Ok(true)
        }
    } else {
        Ok(true)
    }
}

async fn scroll(page: &WebDriver, direction: ScrollDirection ) -> Result<(), CustomError> {
    let mut x = 0;

    while x < 30 {
        move_scroll(page,&direction).await?;
        x += 1;
    }

    Ok(())
}
enum ScrollDirection {
    Top,
    Bottom,
}
async fn move_scroll(page: &WebDriver, direction: &ScrollDirection ) -> Result<(), CustomError> {
    let scroll_code = match direction {
        ScrollDirection::Top => r#"
        let currentPosition = window.scrollY; 
        let scrollDistance = 365;
        let newPosition = currentPosition - scrollDistance; 
        window.scrollTo(0, newPosition); 
    "#,
        ScrollDirection::Bottom => r#"
        let scrollDistance = 365;
        window.scrollBy(0, scrollDistance);
    "#,
    };
    
    page.execute(scroll_code, vec![]).await?;

    Ok(())
}
async fn move_scroll_top(page: &WebDriver) -> Result<(), CustomError> {
    let scroll_code =  r#"
        window.scrollBy(0, 0);
    "#;
    
    page.execute(scroll_code, vec![]).await?;

    wait(1, 1);
    Ok(())
}

fn scrap(html: &str, url_list: &mut Vec<String>) -> Result<(), CustomError> {
    let document = Html::parse_document(html);

    let a_selector = Selector::parse("a[data-test-link-to-profile-link='true']");

    if a_selector.is_err() {
        return Err(CustomError::ButtonNotFound(
            "a_selector is error/Scrap Recruiter Search".to_string(),
        ));
    }
    let a_selector = a_selector.unwrap();

    let linkedin_urls: Vec<String> = document
        .select(&a_selector)
        .filter_map(|el| el.value().attr("href"))
        .map(String::from)
        .collect();

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
