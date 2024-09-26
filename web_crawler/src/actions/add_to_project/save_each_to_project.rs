use crate::actions::scrap_profile::scrap_each_profile::find_entity_urn;
use crate::actions::scrap_recruiter_search::check_recruiter_cookie;

use crate::actions::init_browser::session_cookie_is_valid;
use crate::actions::wait::wait;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};
use thirtyfour::{By, WebDriver};

pub async fn save_each(
    browser: &WebDriver,
    candidate_linkedin: &str,
    project_name: &str,
) -> Result<(), CustomError> {
    browser.goto(candidate_linkedin).await?;
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

    let entity_urn = get_urn(&browser).await?;
    let recruiter_url = format!(
        "https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER",
        entity_urn
    );
    browser.goto(&recruiter_url).await?;
    wait(20, 23);
    let recruiter_session_cookie_check = check_recruiter_cookie(&browser).await?;
    if !recruiter_session_cookie_check {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = check_recruiter_cookie(&browser).await?;
        if !cookie_second_try {
            return Err(CustomError::RecruiterSessionCookieExpired);
        }
    }

    save_project_button(&browser).await?;
    existing_project_boolean(&browser).await?;
    search_project_input(&browser, project_name).await?;
    wait(4, 7);
    click_project_in_dropdown(browser, project_name).await?;
    wait(3, 5);
    final_save(browser).await?;
    Ok(())
}
async fn final_save(browser: &WebDriver) -> Result<(), CustomError> {
    const SAVE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--pro.artdeco-button--primary.ember-view.hp-core-save-to-project__action";
    let save_button = browser.find(By::Css(SAVE_BUTTON)).await;

    let button = match save_button {
        Ok(button) => button,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Final Save to project button not found".to_string(),
            ));
        }
    };

    const ALERT: &str = "div[class='hp-core-save-to-project__warning-text t-14 t-bold']";
    let alert = browser.find(By::Css(ALERT)).await;

    let _ = match alert {
        Ok(_) => {
            println!("Already exist");
            return Ok(());
        }
        Err(_) => (),
    };
    button.click().await?;
    wait(3, 5);
    Ok(())
}
async fn click_project_in_dropdown(browser: &WebDriver, project: &str) -> Result<(), CustomError> {
    const LIST_SELECTOR: &str = "ul.artdeco-typeahead__results-list.save-to-project-projects-pill-typeahead__result-list.ember-view";
    let list_selector = browser.find(By::Css(LIST_SELECTOR)).await;

    let list_selector = match list_selector {
        Ok(selector) => selector,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Project selector not found".to_string(),
            ));
        }
    };
    let html = list_selector.inner_html().await?;
    let id = find_project_id_in_list(html.as_str(), project)?;
    println!("Project ID: {}", id);
    let project_selector = format!("li[id='{}']", id);
    let list_selector = browser.find(By::Css(&project_selector)).await;

    let list_selector = match list_selector {
        Ok(selector) => selector,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Project with specific id not found".to_string(),
            ));
        }
    };
    list_selector.click().await?;
    Ok(())
}

fn find_project_id_in_list(html: &str, project: &str) -> Result<String, CustomError> {
    let document = Html::parse_document(html);
    //println!("document: {:?}", html);
    // Define the selectors
    //let ul_selector = Selector::parse("ul.artdeco-typeahead__results-list.save-to-project-projects-pill-typeahead__result-list.ember-view").unwrap();
    let li_selector = Selector::parse("li.artdeco-typeahead__result.ember-view").unwrap();
    let project_name_selector = Selector::parse("span.project-lockup-title__item.t-bold").unwrap();

    // Find the desired project and click on it
    //for ul in document.select(&ul_selector) {
    for li in document.select(&li_selector) {
        //println!("li: {:?}", li);
        let be = li.select(&project_name_selector).next();
        //println!("be: {:?}", be);
        if let Some(project_name) = li.select(&project_name_selector).next() {
            println!(
                "project_name: {:?});",
                project_name.text().collect::<Vec<_>>().concat()
            );
            if project_name.text().collect::<Vec<_>>().concat().trim() == project {
                let li_html = li.html();

                let li_id = li.value().id().unwrap();
                println!("li_id: {:?}", li_id);
                return Ok(li_id.to_string());
            }
        }
    }
    //}

    return Err(CustomError::ButtonNotFound(
        "Project ID is not found  not found".to_string(),
    ));
}
async fn search_project_input(browser: &WebDriver, project: &str) -> Result<(), CustomError> {
    const SEARCH_INPUT: &str = "input[id=save-to-projects-typeahead]";
    let search_input = browser.find(By::Css(SEARCH_INPUT)).await;

    let search_input = match search_input {
        Ok(input) => input,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Search project input not found".to_string(),
            ));
        }
    };
    search_input.focus().await?; // focus on input for note
    wait(1, 2); // random dela
    search_input.send_keys(project).await?;
    wait(3, 5);
    Ok(())
}
async fn save_project_button(browser: &WebDriver) -> Result<(), CustomError> {
    const SAVE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--pro.artdeco-button--secondary.ember-view.profile-item-actions__item";
    let save_button = browser.find(By::Css(SAVE_BUTTON)).await;

    let button = match save_button {
        Ok(button) => button,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Save to project button not found".to_string(),
            ));
        }
    };
    button.click().await?;
    wait(3, 5);
    Ok(())
}
async fn existing_project_boolean(browser: &WebDriver) -> Result<(), CustomError> {
    const BOOLEAN: &str = "label[for=choose-existing-projects]";
    let boolean = browser.find(By::Css(BOOLEAN)).await;
    let boolean = match boolean {
        Ok(boolean) => boolean,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Existing project boolean not found".to_string(),
            ));
        }
    };
    boolean.click().await?;
    wait(3, 5);
    Ok(())
}
async fn get_urn(browser: &WebDriver) -> Result<String, CustomError> {
    const HTML_BODY: &str =
        "body.render-mode-BIGPIPE.nav-v2.ember-application.icons-loaded.boot-complete";
    let html_body = browser.find(By::Css(HTML_BODY)).await;

    let html = match html_body {
        Ok(body) => body.inner_html().await?,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Body er is not found".to_string(),
            ));
        }
    };
    let urn = find_entity_urn(&html);
    let urn = match urn {
        Some(urn) => urn,
        None => {
            return Err(CustomError::ButtonNotFound(
                "Entity urn is not found".to_string(),
            ));
        }
    };

    Ok(urn)
}
