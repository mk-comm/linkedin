use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::connection::Connection;
use crate::structs::entry::EntryScrapConnection;
use crate::structs::error::CustomError;
use crate::structs::fullname::FullName;
use scraper::{Html, Selector};
use serde_json::json;

use crate::actions::start_browser::send_screenshot;
pub async fn scrap_connections(entry: EntryScrapConnection) -> Result<(), CustomError> {
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
    let user_id = browser_info.user_id.clone();
    let browser = start_browser(browser_info).await?;

    wait(7, 12);

    let my_network_button = browser
        .page
        .query_selector("a[href='https://www.linkedin.com/mynetwork/?']")
        .await?;

    match my_network_button {
        Some(button) => {
            button.hover_builder();
            wait(1, 3);
            button.click_builder().click().await?;
            wait(1, 3);
        }
        None => {
            let screenshot = browser.page.screenshot_builder().screenshot().await?;

            send_screenshot(
                screenshot,
                &user_id,
                "Network button is missing",
                "Scrap connection",
            )
            .await?;
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Network button is missing".to_string(),
            ));
        }
    }

    wait(12, 17);

    const URL: &str = "https://www.linkedin.com/mynetwork/invite-connect/connections/";

    let button = browser
        .page
        .query_selector("div.mn-community-summary__entity-info")
        .await?;
    match button {
        Some(button) => {
            button.hover_builder();
            wait(1, 3);
            button.click_builder().click().await?;
            wait(8, 12);
        }
        None => {
            let build = browser.page.goto_builder(URL).goto().await;
            match build {
                Ok(_) => (),
                Err(_) => {
                    return Err(CustomError::ButtonNotFound(
                        "Connection page is not opening".to_string(),
                    ))
                }
            }
            wait(11, 16);
            /*
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound(
                "Connections button is missing".to_string(),
            ));
            */
        }
    }

    let connections = scrap_each_connection(&browser.page.content().await?);
    if connections.len() == 0 {
        browser.page.close(Some(false)).await?;
        browser.browser.close().await?; // close browser
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
            .json(&payload)
            .send()
            .await;
    }

    browser.page.close(Some(false)).await?;
    browser.browser.close().await?; // close browser
    Ok(())
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
