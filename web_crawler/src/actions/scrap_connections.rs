use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::entry::{Entry, EntryRegular};
use crate::structs::error::CustomError;
use crate::structs::browser::BrowserInit;


#[allow(dead_code)] // delete later
pub async fn scrap_connections(entry: Entry) -> Result<(), CustomError> {
    #[allow(dead_code)] // delete later
    //let api_key = entry.user_id.clone();

    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: None,
        };

    let browser = start_browser(browser_info).await?;

    wait(3, 7);

    let my_network_button = browser
        .page
        .query_selector("div.global-nav__primary-link-notif.artdeco-notification-badge.ember-view")
        .await?;

    match my_network_button {
        Some(button) => {
            button.hover_builder();
            wait(1, 3);
            button.click_builder().click().await?;
            wait(1, 3);
            println!("my network button is ok")
        }
        None => {
            println!("my network button is not ok");
        }
    }

    let button = browser
        .page
        .wait_for_selector_builder("div.mn-community-summary__entity-info")
        .wait_for_selector()
        .await?;
    match button {
        Some(button) => {
            button.hover_builder();
            wait(1, 3);
            button.click_builder().click().await?;
            wait(1, 3);
        }
        None => {
            println!("button is not ok");
        }
    }
    wait(10, 20);

    Ok(())
}
