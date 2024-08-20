use crate::actions::scrap_profile::scrap_each_profile::find_entity_urn;
use crate::actions::wait::wait;
use crate::structs::browser::{BrowserConfig, BrowserInit};
use crate::structs::error::CustomError;
use scraper::{Html, Selector};

pub async fn save_each(
    browser: &BrowserConfig,
    candidate_linkedin: &str,
    project_name: &str,
) -> Result<(), CustomError> {
    browser.page.goto_builder(candidate_linkedin).goto().await?;
    wait(7, 10);
    let entity_urn = get_urn(&browser).await?;
    let recruiter_url = format!(
        "https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER",
        entity_urn
    );
    browser.page.goto_builder(&recruiter_url).goto().await?;
    wait(7, 10);

    save_project_button(&browser).await?;
    existing_project_boolean(&browser).await?;
    search_project_input(&browser, project_name).await?;
    wait(4, 7);
    click_project_in_dropdown(browser, project_name).await?;
    wait(3, 5);
    final_save(browser).await?;
    Ok(())
}
async fn final_save(browser: &BrowserConfig) -> Result<(), CustomError> {
    const SAVE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--pro.artdeco-button--primary.ember-view.hp-core-save-to-project__action";
    let save_button = browser.page.query_selector(SAVE_BUTTON).await?;
    let button = match save_button {
        Some(button) => button,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Final Save to project button not found".to_string(),
            ));
        }
    };

    const ALERT: &str = "div[class='hp-core-save-to-project__warning-text t-14 t-bold']";
    let alert = browser.page.query_selector(ALERT).await?;

    let _ = match alert {
        Some(_) => {
            println!("Already exist");
            return Ok(());
        }
        None => (),
    };
    //wait(100000, 500000); // random delay
    button.hover_builder(); // hover on search input
    wait(1, 4); // random delay
    button.click_builder().click().await?;
    wait(3, 5);
    Ok(())
}
async fn click_project_in_dropdown(
    browser: &BrowserConfig,
    project: &str,
) -> Result<(), CustomError> {
    const LIST_SELECTOR: &str = "ul.artdeco-typeahead__results-list.save-to-project-projects-pill-typeahead__result-list.ember-view";
    let list_selector = browser.page.query_selector(LIST_SELECTOR).await?;
    let list_selector = match list_selector {
        Some(selector) => selector,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Project selector not found".to_string(),
            ));
        }
    };
    let html = list_selector.inner_html().await?;
    let id = find_project_id_in_list(html.as_str(), project)?;
    println!("Project ID: {}", id);
    let project_selector = format!("li[id='{}']", id);
    let list_selector = browser
        .page
        .query_selector(project_selector.as_str())
        .await?;
    let list_selector = match list_selector {
        Some(selector) => selector,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Project with specific id not found".to_string(),
            ));
        }
    };
    list_selector.hover_builder();
    wait(1, 5); // random delay
    list_selector.click_builder().click().await?;
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
async fn search_project_input(browser: &BrowserConfig, project: &str) -> Result<(), CustomError> {
    const SEARCH_INPUT: &str = "input[id=save-to-projects-typeahead]";
    let search_input = browser.page.query_selector(SEARCH_INPUT).await?;
    let search_input = match search_input {
        Some(input) => input,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Search project input not found".to_string(),
            ));
        }
    };
    search_input.focus().await?; // focus on input for note
    wait(1, 2); // random delay
    search_input.fill_builder(project).fill().await?;
    wait(3, 5);
    Ok(())
}
async fn save_project_button(browser: &BrowserConfig) -> Result<(), CustomError> {
    const SAVE_BUTTON: &str = "button.artdeco-button.artdeco-button--2.artdeco-button--pro.artdeco-button--secondary.ember-view.profile-item-actions__item";
    let save_button = browser.page.query_selector(SAVE_BUTTON).await?;
    let button = match save_button {
        Some(button) => button,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Save to project button not found".to_string(),
            ));
        }
    };
    button.hover_builder(); // hover on search input
    wait(1, 4); // random delay
    button.click_builder().click().await?;
    wait(3, 5);
    Ok(())
}
async fn existing_project_boolean(browser: &BrowserConfig) -> Result<(), CustomError> {
    const BOOLEAN: &str = "label[for=choose-existing-projects]";
    let boolean = browser.page.query_selector(BOOLEAN).await?;
    let boolean = match boolean {
        Some(boolean) => boolean,
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Existing project boolean not found".to_string(),
            ));
        }
    };
    boolean.hover_builder(); // hover on search input
    wait(1, 4); // random delay
    boolean.click_builder().click().await?;
    wait(3, 5);
    Ok(())
}
async fn get_urn(browser: &BrowserConfig) -> Result<String, CustomError> {
    let html_body = browser
        .page
        .query_selector(
            "body.render-mode-BIGPIPE.nav-v2.ember-application.icons-loaded.boot-complete",
        )
        .await?;
    let html = match html_body {
        Some(body) => body.inner_html().await?,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Body er is not found".to_string(),
            ));
        }
    };
    let urn = find_entity_urn(&html);
    let urn = match urn {
        Some(urn) => urn,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "Entity urn is not found".to_string(),
            ));
        }
    };

    Ok(urn)
}
