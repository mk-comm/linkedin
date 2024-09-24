use crate::actions::init_browser::{init_browser, send_screenshot, session_cookie_is_valid};
use crate::actions::wait::wait;

use crate::structs::browser::BrowserInit;
use crate::structs::connection::Connection;
use crate::structs::entry::EntryScrapConnection;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use scraper::{Html, Selector};
use serde_json::json;
use thirtyfour::{By, WebDriver};

pub async fn scrap_connections(entry: EntryScrapConnection) -> Result<String, CustomError> {
    let api_key = entry.user_id.clone();

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
    let result = scrap(&browser, &api_key).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &browser_info.user_id,
                text.as_str(),
                &browser_info.user_id,
                "Scrap connections",
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
                &browser_info.user_id,
                "Scrap connections",
            )
            .await?;
            return Err(error);
        }
    }
}
pub async fn scrap(browser: &WebDriver, api_key: &str) -> Result<String, CustomError> {
    const NETWORK_TAB: &str = "https://www.linkedin.com/mynetwork/invite-connect/connections/";
    let go_to = browser.goto(NETWORK_TAB).await;

    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser.goto(NETWORK_TAB).await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                return Err(CustomError::ButtonNotFound(
                    "Network page is not loading/Scrap connections".to_string(),
                ));
            }
            x += 1;
        }
        wait(1, 3);
    }
    wait(12, 17);
    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            wait(1, 3);
            return Err(CustomError::SessionCookieExpired);
        }
    }
    const MAIN_FRAME: &str = "div.scaffold-finite-scroll__content";
    let main_frame = browser.find(By::Css(MAIN_FRAME)).await;
    let main_frame = match main_frame {
        Ok(list) => list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Project list not found/Change stage".to_string(),
            ));
        }
    };

    let connections = scrap_each_connection(&main_frame.inner_html().await?);
    if connections.len() == 0 {
        return Err(CustomError::ButtonNotFound(
            "Connections vec is zero, double check".to_string(),
        ));
    }

    for connection in connections {
        wait(1, 1);
        let client = reqwest::Client::new();
        let payload = json!({
        "full_name": connection.full_name,
        "first_name": connection.first_name,
        "last_name": connection.last_name,
        "profile_url": connection.profile_url,
        "api_key": api_key,
        });
        let _res = client
            .post("https://overview.tribe.xyz/api/1.1/wf/tribe_api_connections")
            //.post("https://webhook.site/c7bac898-bab7-4a36-8874-9dbcf72c01f5")
            .json(&payload)
            .send()
            .await;
    }

    Ok("Connections was scraped".to_string())
}

fn scrap_each_connection(html: &str) -> Vec<Connection> {
    let mut connections = Vec::new();

    let document = Html::parse_document(html);

    // Create a selector for the li elements, which represent each connection
    let li_selector = Selector::parse("li.mn-connection-card").unwrap();

    // Create selectors for each piece of data you're interested in
    let name_selector = Selector::parse("span.mn-connection-card__name").unwrap();
    let url_selector = Selector::parse("a.mn-connection-card__link").unwrap();

    for li in document.select(&li_selector) {
        let name_elem = li.select(&name_selector).next().unwrap();

        let full_name = FullName::split_name(name_elem.inner_html().trim());
        let url_elem = li.select(&url_selector).next().unwrap();
        let profile_url = url_elem.value().attr("href").unwrap().to_string();

        //println!("First Name: {:?}", full_name);

        //println!("Profile URL: https://www.linkedin.com{}", profile_url);

        let connection = Connection {
            first_name: full_name.first_name,
            last_name: full_name.last_name,
            full_name: full_name.full_name,
            profile_url: format!("https://www.linkedin.com{}", profile_url),
        };
        connections.push(connection);
    }

    connections
}
