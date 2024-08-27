use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::{BrowserConfig, BrowserInit};
use crate::structs::entry::EntryScrapProjects;
use crate::structs::error::CustomError;
use playwright::api::ElementHandle;
use playwright::api::Page;
use serde::Serialize;
use serde_json::json;
use tracing::{error, info};

use scraper::{Html, Selector};
pub async fn scrap_projects(entry: EntryScrapProjects) -> Result<(), CustomError> {
    let target_url = entry.target_url.clone();
    let user_id = entry.user_id.clone();
    let browser = init(entry).await?;
    open_list_projects(&browser).await?;
    //clear_all(&browser).await?;
    loop {
        scroll(&browser.page).await?;
        let container = find_list_container(&browser).await?;
        let projects = scrap_list(container.inner_html().await?.as_str())?;
        send_urls(projects, &target_url, &user_id).await?;
        let next_button = find_next_button(&browser).await;
        if next_button.is_err() {
            println!("next is not found");
            break;
        } else {
            wait(11, 13);
            move_scroll_top(&browser.page).await?;
        }
    }
    Ok(())
}

async fn init(entry: EntryScrapProjects) -> Result<BrowserConfig, CustomError> {
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

    let browser = start_browser(browser_info).await?;
    wait(7, 10); // random delay
    Ok(browser)
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

async fn move_scroll_top(page: &Page) -> Result<(), CustomError> {
    let scroll_code = r#"
    function() {
    window.scrollTo({
        top: 0,
        left: 0,
        behavior: 'smooth'
    });
}
    "#;

    page.evaluate(scroll_code, ()).await?;

    wait(1, 2);
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
async fn find_next_button(browser: &BrowserConfig) -> Result<ElementHandle, CustomError> {
    const ID: &str = "span.pagination__quick-link-icon>li-icon[type=chevron-right-icon]";

    let next_button = browser.page.query_selector(ID).await?;
    match next_button {
        Some(button) => {
            button.click_builder().click().await?;
            Ok(button)
        }
        None => {
            return Err(CustomError::ButtonNotFound(
                "Next button is not found".to_string(),
            ))
        }
    }
}

async fn find_list_container(browser: &BrowserConfig) -> Result<ElementHandle, CustomError> {
    const ID: &str = "ol.ember-view.projects-list-layout__list";

    let list_container = browser.page.query_selector(ID).await?;
    match list_container {
        Some(container) => Ok(container),
        None => {
            return Err(CustomError::ButtonNotFound(
                "Container list is not found".to_string(),
            ))
        }
    }
}

async fn clear_all(browser: &BrowserConfig) -> Result<(), CustomError> {
    let clear_all_button = browser
        .page
        .query_selector("button.artdeco-button.artdeco-button--muted.artdeco-button--1.artdeco-button--tertiary.ember-view")
        .await?;
    match clear_all_button {
        Some(button) => {
            button.hover_builder();
            wait(1, 3);
            button.click_builder().click().await?;
            wait(9, 12);
        }
        None => (),
    };
    Ok(())
}

async fn open_list_projects(browser: &BrowserConfig) -> Result<(), CustomError> {
    const URL: &str = "https://www.linkedin.com/talent/projects";
    let _projects_page = browser.page.goto_builder(URL).goto().await?;
    wait(10, 14);
    Ok(())
}

fn scrap_list(body: &str) -> Result<Vec<Project>, CustomError> {
    let document = Html::parse_document(&body);
    let mut projects = Vec::new();
    let project_selector = Selector::parse("li[data-test-paginated-list-item]").unwrap();
    let name_selector = Selector::parse("h2.project-list-item__name > div > span").unwrap();
    let url_selector = Selector::parse("a.project-list-item__project-card-link").unwrap();
    let arhived_selector =
        Selector::parse("span.project-lockup-title__item.t-12.t-black--light").unwrap();

    for project_element in document.select(&project_selector) {
        let name = project_element
            .select(&name_selector)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default()
            .trim()
            .to_string();

        let id = project_element
            .select(&url_selector)
            .next()
            .map(|el| el.value().attr("href").unwrap_or_default().to_string())
            .unwrap_or_default()
            .replace("/talent/hire/", "")
            .replace("/overview", "")
            .trim()
            .to_string();

        let archived_element = project_element.select(&arhived_selector).next();
        let archived = match archived_element {
            Some(_) => true,
            None => false,
        };
        projects.push(Project { name, id, archived });
    }

    // Print or process the projects
    for project in &projects {
        println!("{:?}", project);
    }

    // Optionally, serialize the projects to JSON and print
    let projects_json = serde_json::to_string_pretty(&projects)?;
    println!("{}", projects_json);

    Ok(projects)
}

#[derive(Debug, Serialize)]
struct Project {
    name: String,
    id: String,
    archived: bool,
}

async fn send_urls(
    projects: Vec<Project>,
    target_url: &str,
    user_id: &str,
) -> Result<(), CustomError> {
    let max_retries = 5;
    let client = reqwest::Client::new();

    for batch in projects.chunks(10) {
        let urls_json = json!({
            "projects": batch,
            "user_id": user_id
        });

        let mut retries = 0;
        loop {
            let response = client.post(target_url).json(&urls_json).send().await;
            match response {
                Ok(res) => {
                    if res.status() == 200 {
                        info!(
                            "Send_urls/scrap_linkedin_projects/Ok: {}, status: {}/URL {}",
                            user_id,
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
                                "Send_urls/scrap_linkedin_projects/Error {}: status {}/URL: {}",
                                user_id,
                                res.status(),
                                target_url
                            );
                            return Err(CustomError::ButtonNotFound(
                                "Send url status is not 200, Status/Scrap Linkedin Projects"
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
                        error!(error = ?error, "Send_urls/scrap_linkedin_projects/Error {} returned error {}/URL: {}", user_id, error, target_url);

                        return Err(CustomError::ButtonNotFound(
                            "Scrap Linkedin Projects send url, Error".to_string(),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
