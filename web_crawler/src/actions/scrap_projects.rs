use crate::actions::init_browser::{init_browser, send_screenshot};
use crate::actions::scrap_recruiter_search::check_recruiter_cookie;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapProjects;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::json;
use thirtyfour::{By, WebDriver, WebElement};
use tracing::{error, info};
#[derive(Debug, Serialize)]
struct Project {
    name: String,
    id: String,
    archived: bool,
    order: u32,
}
pub async fn scrap_projects(entry: EntryScrapProjects) -> Result<String, CustomError> {
    let target_url = entry.target_url.clone();
    let user_id = entry.user_id.clone();
    let browser = init(entry).await?;
    let result = run(&browser, &target_url, &user_id).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                text.as_str(),
                &user_id,
                "Scrap projects",
            )
            .await?;
            browser.quit().await?;
            return Ok(text);
        }
        Err(error) => {
            let screenshot = browser.screenshot_as_png().await?;
            browser.quit().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Scrap projects",
                &user_id,
                "Scrap projects",
            )
            .await?;
            return Err(error);
        }
    }
}

async fn run(browser: &WebDriver, target_url: &str, user_id: &str) -> Result<String, CustomError> {
    open_list_projects(browser).await?;
    wait(10, 15);
    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::RecruiterSessionCookieExpired);
        }
    }
    let zoom_script = "document.body.style.zoom = '50.0%';";
    browser.execute(&zoom_script, vec![]).await?;
    let mut order = 0;
    loop {
        wait(2, 4);
        scroll(&browser).await?;
        let container = find_list_container(&browser).await?;
        let projects = scrap_list(container.inner_html().await?.as_str(), &mut order)?;
        for project in &projects {
            if project.name.is_empty() {
                let screenshot = browser.screenshot_as_png().await?;
                send_screenshot(
                    screenshot,
                    &user_id,
                    "Each page",
                    &user_id,
                    "Scrap projects",
                )
                .await?;
            }
        }
        send_urls(projects, &target_url, &user_id).await?;
        let next_button = find_next_button(&browser).await;
        if next_button.is_err() {
            println!("next is not found");
            break;
        } else {
            wait(11, 13);
            move_scroll_top(&browser).await?;
        }
    }

    Ok("Scraping projects finished successfully".to_string())
}

async fn init(entry: EntryScrapProjects) -> Result<WebDriver, CustomError> {
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
    wait(7, 10); // random delay
    Ok(browser)
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

async fn move_scroll_top(page: &WebDriver) -> Result<(), CustomError> {
    let scroll_code = r#"
    (function() {
        window.scrollTo({
            top: 0,
            left: 0,
            behavior: 'smooth'
        });
    })();
"#;

    page.execute(scroll_code, vec![]).await?;

    wait(1, 2);
    Ok(())
}

async fn scroll(page: &WebDriver) -> Result<(), CustomError> {
    let mut x = 0;

    while x < 25 {
        move_scroll(page).await?;
        x += 1;
    }

    Ok(())
}
async fn find_next_button(browser: &WebDriver) -> Result<WebElement, CustomError> {
    const ID: &str = "span.pagination__quick-link-icon>li-icon[type=chevron-right-icon]";
    const NEXT: &str =
        "href.pagination__quick-link.pagination__quick-link--next.link-without-hover-visited";
    let next_button = browser.find(By::Css(ID)).await;
    match next_button {
        Ok(button) => {
            button.click().await?;
            Ok(button)
        }
        Err(_) => match browser.find(By::Css(NEXT)).await {
            Ok(button) => {
                button.click().await?;
                Ok(button)
            }
            Err(_) => {
                return Err(CustomError::ButtonNotFound(
                    "Next button is not found".to_string(),
                ))
            }
        },
    }
}

async fn find_list_container(browser: &WebDriver) -> Result<WebElement, CustomError> {
    const ID: &str = "ol.ember-view.projects-list-layout__list";

    let list_container = browser.find(By::Css(ID)).await;
    match list_container {
        Ok(container) => Ok(container),
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Container list is not found".to_string(),
            ))
        }
    }
}

async fn open_list_projects(browser: &WebDriver) -> Result<(), CustomError> {
    const URL: &str = "https://www.linkedin.com/talent/projects";
    let _projects_page = browser.goto(URL).await?;
    wait(10, 14);
    Ok(())
}

fn scrap_list(body: &str, order: &mut u32) -> Result<Vec<Project>, CustomError> {
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
        let name = name.replace("<!---->", "").trim().to_string();

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
        projects.push(Project {
            name,
            id,
            archived,
            order: *order,
        });
        *order += 1;
    }
    let projects_json = serde_json::to_string_pretty(&projects)?;
    println!("{}", projects_json);

    Ok(projects)
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
