use crate::actions::wait::wait;
use crate::structs::error::CustomError;
use scraper::{Html, Selector};

use thirtyfour::{By, WebDriver};
pub async fn change_stage(browser: &WebDriver, project_name: &str) -> Result<String, CustomError> {
    let html_project_list = projects_list(&browser).await?;
    let id = find_project_id(&html_project_list, project_name)?;
    find_and_click_element_by_id(&browser, &id).await?;
    wait(7, 9);
    let current_stage = current_stage(&browser).await?;
    if current_stage != "uncontacted" {
        return Ok("Candidate in stage above contacted".to_string());
    }
    find_and_click_change_stage_dropdown(browser).await?;
    wait(6, 9);
    find_and_click_contacted_stage(browser).await?;
    //let html_stages_list = find_stages_list(&browser).await?;
    //let selector = find_stage_selector(&html_stages_list, &stage)?;
    //find_and_click_stage_by_selector(&browser, &selector).await?;

    Ok("Candidate was moved to the stage".to_string())
}

async fn projects_list(browser: &WebDriver) -> Result<String, CustomError> {
    const LIST: &str = "div[class='topcard-requisitions topcard-condensed__requisitions']";
    let list = browser.find(By::Css(LIST)).await;

    let list = match list {
        Ok(list) => list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Project list not found/Change stage".to_string(),
            ));
        }
    };
    let html = list.inner_html().await?;

    Ok(html)
}

async fn current_stage(browser: &WebDriver) -> Result<String, CustomError> {
    const STAGE: &str = "p.t-14.t-black--light:has(span.t-black.t-bold) > span.t-black.t-bold";
    let stage = browser.find(By::Css(STAGE)).await;

    let stage = match stage {
        Ok(list) => list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Current stage is not found/Change stage".to_string(),
            ));
        }
    };
    let stage = stage.text().await?;
    println!("CURRENT STAGE {}", stage);
    Ok(stage)
}

fn find_project_id(html_content: &str, project_name: &str) -> Result<String, CustomError> {
    let document = Html::parse_document(html_content);
    let item_block_selector = Selector::parse(r#"div.topcard-requisitions__item-block"#).unwrap();
    let name_selector = Selector::parse(r#"span.topcard-requisitions__name"#).unwrap();
    let link_selector = Selector::parse(r#"a.ember-view"#).unwrap();

    for item_block in document.select(&item_block_selector) {
        if let Some(name_element) = item_block.select(&name_selector).next() {
            if let Some(link_element) = name_element.select(&link_selector).next() {
                let link_text = link_element
                    .text()
                    .collect::<Vec<_>>()
                    .concat()
                    .trim()
                    .to_string();
                if link_text == project_name {
                    if let Some(id) = link_element.value().attr("id") {
                        return Ok(id.to_string());
                    }
                }
            }
        }
    }

    return Err(CustomError::ButtonNotFound(
        "Project not found in the list/Change Stage".to_string(),
    ));
}

async fn find_and_click_element_by_id(
    browser: &WebDriver,
    element_id: &str,
) -> Result<(), CustomError> {
    let link = browser.find(By::Css(&format!("#{}", element_id))).await;

    let link = match link {
        Ok(link) => link,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Can't find link based on url/Change stage".to_string(),
            ));
        }
    };

    link.click().await?;

    Ok(())
}

async fn find_and_click_change_stage_dropdown(browser: &WebDriver) -> Result<(), CustomError> {
    const DROPDOWN: &str = "button[id='requisition-actions_move-to-pipeline']";
    let dropdown = browser.find(By::Css(DROPDOWN)).await;

    let dropdown = match dropdown {
        Ok(dropdown) => dropdown,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Can't find dropdown/Change stage".to_string(),
            ));
        }
    };

    dropdown.click().await?;

    Ok(())
}

async fn _find_stages_list(browser: &WebDriver) -> Result<String, CustomError> {
    const LIST: &str = "ol[class='requisition-pipeline-activity__stages']";
    let list = browser.find(By::Css(LIST)).await;

    let list = match list {
        Ok(list) => list,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Can't find dropdown/Change stage".to_string(),
            ));
        }
    };

    let html = list.inner_html().await?;

    Ok(html)
}

fn _find_stage_selector(html_content: &str, stage_name: &str) -> Result<String, CustomError> {
    println!("html_content: {}", html_content);
    let document = Html::parse_document(html_content);
    let stage_selector =
        Selector::parse("li[class='requisition-pipeline-activity__stage']").unwrap();
    let name_selector = Selector::parse("p[class='t-14.t-black--light']").unwrap();

    for (index, stage) in document.select(&stage_selector).enumerate() {
        if let Some(name_element) = stage.select(&name_selector).next() {
            let name_text = name_element
                .text()
                .collect::<Vec<_>>()
                .concat()
                .trim()
                .to_string();
            println!("name_text: {}", name_text);
            if name_text.trim() == stage_name {
                return Ok(format!(
                    "li.requisition-pipeline-activity__stage:nth-of-type({})",
                    index + 1
                ));
            }
        }
    }
    return Err(CustomError::ButtonNotFound(
        "Can't stage selector/Change stage".to_string(),
    ));
}

async fn _find_and_click_stage_by_selector(
    browser: &WebDriver,
    element_id: &str,
) -> Result<(), CustomError> {
    let link = browser.find(By::Css(&format!("#{}", element_id))).await;

    let link = match link {
        Ok(link) => link,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Can't find stage based on selector/Change stage".to_string(),
            ));
        }
    };

    link.click().await?;

    Ok(())
}

async fn find_and_click_contacted_stage(browser: &WebDriver) -> Result<(), CustomError> {
    const DROPDOWN: &str = "div[class=artdeco-dropdown__content-inner] > ol > li:nth-of-type(2)";
    let dropdown = browser.find(By::Css(DROPDOWN)).await;
    let dropdown = match dropdown {
        Ok(dropdown) => dropdown,
        Err(_) => {
            return Err(CustomError::ButtonNotFound(
                "Can't find contacted dropdown/Change stage".to_string(),
            ));
        }
    };
    //const ANOTHER_DROPDOWN: &str = "div[class=artdeco-dropdown__content-inner]";
    //let another_dropdown = browser.find(By::Css(ANOTHER_DROPDOWN)).await;

    dropdown.click().await?;

    Ok(())
}
