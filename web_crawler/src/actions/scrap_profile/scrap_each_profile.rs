use crate::actions::init_browser::session_cookie_is_valid;
use crate::actions::scrap_profile::misc::serialize_option_string;
use crate::actions::scrap_profile::scrap_education::parse_education;
use crate::actions::scrap_profile::scrap_education::Education;
use crate::actions::scrap_profile::scrap_experience_new_tab::{parse_experience, Experience};
use crate::structs::entry::Url;
use crate::structs::error::CustomError;
use playwright::api::browser;
use serde::{Deserialize, Serialize};
use thirtyfour::{By, WebDriver};

use crate::actions::start_browser::send_screenshot;
use crate::actions::wait::wait;
#[allow(deprecated)]
use base64::encode;
use scraper::{Html, Selector};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ResultJson {
    b64: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct BodyJsonB64 {
    body: ResultJson,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub AI: bool,
    #[serde(serialize_with = "serialize_option_string")]
    pub linkedin: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub first: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub last: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub email: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub job: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub sourcer: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub title: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub linkedin_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub linkedin_unique_number: Option<String>,
    pub connectionLevel: Option<i32>,
    #[serde(serialize_with = "serialize_option_string")]
    pub company: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub company_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub about: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub profilePicture: Option<String>,
    pub education: Vec<Education>,
    pub experience: Vec<Experience>,
    #[serde(serialize_with = "serialize_option_string")]
    pub viewedIn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub location: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub entityUrn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub extension_version: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub timestamp: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub search_url: Option<String>,
    pub profile_url_id: String,
}

pub async fn scrap_each_profile_main(
    browser: &WebDriver,
    job: Option<String>,
    sourcer: Option<String>,
    aisearch: Option<String>,
    search_url: Option<String>,
    url_id: String,
) -> Result<Profile, CustomError> {
    const PAGE_NOT_FOUND: &str = "header[class='not-found__header not-found__container']";
    let page_not_found = browser.find(By::Css(PAGE_NOT_FOUND)).await;

    if page_not_found.is_ok() {
        let screenshot = browser.screenshot_as_png().await?;
        send_screenshot(
            screenshot,
            "aisearch",
            "Page does not exist",
            "scrap each profile",
        )
        .await?;
        return Err(CustomError::ButtonNotFound(
            "Page does not exist".to_string(),
        ));
    }
    let cookie = session_cookie_is_valid(&browser).await?;
    if !cookie {
        browser.refresh().await?;
        wait(7, 14);
        let cookie_second_try = session_cookie_is_valid(&browser).await?;
        if !cookie_second_try {
            wait(1, 3);
            let screenshot = browser.screenshot_as_png().await?;
            browser.clone().quit().await?;
            send_screenshot(
                screenshot,
                "Scrap each profile",
                "Session cookie expired",
                "Scrap each profile",
            )
            .await?;
            return Err(CustomError::SessionCookieExpired);
        }

        println!("checking if cookie is valid{}", cookie_second_try);
    }
    const HTML_BODY: &str =
        "body.render-mode-BIGPIPE.nav-v2.ember-application.icons-loaded.boot-complete";
    let html_body = browser.find(By::Css(HTML_BODY)).await;

    let html = match html_body {
        Ok(body) => body.inner_html().await?,
        Err(_) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                "aisearch",
                "Body HTML is not found",
                "scrap each profile",
            )
            .await?;
            return Err(CustomError::ButtonNotFound(
                "Body HTML is not found".to_string(),
            ));
        }
    };
    let linkedin_url = browser.current_url().await?;
    let linkedin_url = linkedin_url.as_str().to_string();
    let full_name = get_name_tuple(&html);
    let first_name = match full_name {
        Some(ref full_name) => Some(full_name.clone().0),
        None => None,
    };
    let last_name = match full_name {
        Some(ref full_name) => Some(full_name.clone().1),
        None => None,
    };
    let current_tittle = get_title(&html);
    let linkedin_nick = get_linkedin_nick(Some(linkedin_url.clone()));
    let linkedin_main_id = get_linkedin_main_id(&html);
    let profile_picture = get_profile_picture(&html);
    let about = get_about(&html);
    let connection_level = Some(5);
    let location = get_location(&html);
    let entity_urn = find_entity_urn(&html);

    let education = parse_education(&html);
    let profile = Profile {
        AI: true,
        linkedin: Some(linkedin_url),
        first: first_name,
        last: last_name,
        email: None,
        job,
        sourcer,
        title: current_tittle,
        linkedin_unique: linkedin_nick,
        linkedin_unique_number: linkedin_main_id,
        connectionLevel: connection_level,
        company: None,
        company_unique: None,
        about,
        profilePicture: profile_picture,
        education,
        experience: vec![],
        viewedIn: None,
        location,
        entityUrn: entity_urn,
        extension_version: None,
        timestamp: None,
        search_url: search_url.clone(),
        profile_url_id: url_id.to_owned(),
    };

    Ok(profile)
}

pub async fn send_search_status(
    status: &str,
    ai_search: &Option<String>,
    batch_id: &str,
    user_id: &str,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    // Convert the Vec<String> into a JSON string
    let urls_json = json!({
        "status": status,
        "batch_id": *batch_id,
        "ai_search": ai_search,
        "user_id": user_id,
    });
    let target_url = "https://overview.tribe.xyz/api/1.1/wf/search_profile_logs";
    let response: Result<reqwest::Response, reqwest::Error> =
        client.post(target_url).json(&urls_json).send().await;
    match response {
        Ok(_) => info!(
            "Send_search_status/scrap_recruiter_search/Ok, {:?} was done",
            ai_search
        ),
        Err(error) => {
            error!(error = ?error, "Send_search_status/scrap_recruiter_search/Error {:?} returned error {}", ai_search, error);
        }
    }

    Ok(())
}

pub fn find_entity_urn(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a").unwrap();

    // Iterate over all <a> tags to find the URN
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            if let Some(urn) = extract_urn_from_href(href) {
                return Some(
                    urn.replace("urn%3Ali%3Afsd_profile%3A", "")
                        .replace("urn%3Ali%3Afs_normalized_profile%3A", ""),
                );
            }
        }
    }

    // Fallback if no URN found in <a> tags
    get_entity_urn_from_id_attribute(html)
}

fn extract_urn_from_href(href: &str) -> Option<String> {
    //println!("href{}", href);
    // Check for URN in the href attribute
    href.split("?profileUrn=").find_map(|part| {
        if part.starts_with("urn%3Ali%3A") {
            part.split('&').next().map(|s| s.to_string())
        } else {
            None
        }
    })
}

fn get_entity_urn_from_id_attribute(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let id_selector = Selector::parse("[id]").unwrap();

    // Iterate over elements with 'id' attribute
    document.select(&id_selector).find_map(|element| {
        if let Some(id_attr) = element.value().attr("id") {
            if id_attr.contains("datalet")
                && element
                    .html()
                    .contains("/voyager/api/identity/dash/profile")
            {
                extract_urn_from_element_html(element.html().as_str())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn extract_urn_from_element_html(html: &str) -> Option<String> {
    html.find("\"*elements\":[\"urn:li:fsd_profile:")
        .and_then(|start| {
            let start = start + "\"*elements\":[\"urn:li:fsd_profile:".len();
            html[start..]
                .find("\"")
                .map(|end| html[start..start + end].to_string())
        })
}
fn get_location(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let location_selector =
        Selector::parse("span[class='text-body-small inline t-black--light break-words']");
    let location_selector = match location_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    let mut location = String::new();

    for element in document.select(&location_selector) {
        location = element.text().collect::<Vec<_>>().join(" ");
    }
    location = location.trim().to_string();
    Some(location)
}

fn get_about(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let about_selector = Selector::parse("div[class='display-flex ph5 pv3']");

    let about_selector = match about_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    let span_selector = Selector::parse("span[aria-hidden=true]");

    let span_selector = match span_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    document.select(&about_selector).next().and_then(|element| {
        let mut spans = element.select(&span_selector);
        spans
            .next()
            .map(|span| span.text().collect::<Vec<_>>().join(""))
    })
}
fn get_profile_picture(html_content: &str) -> Option<String> {
    let document = Html::parse_fragment(html_content);
    let selector = Selector::parse("div.pv-top-card__non-self-photo-wrapper button img").unwrap();

    // Attempt to find the img element and extract the 'src' attribute
    document
        .select(&selector)
        .next()
        .and_then(|img| img.value().attr("src").map(String::from))
}

fn get_linkedin_main_id(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let id_selector = Selector::parse("section.artdeco-card[data-member-id]");
    let id_selector = match id_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    let id = document
        .select(&id_selector)
        .next()
        // Extract the 'data-member-id' attribute if it exists
        .and_then(|element| element.value().attr("data-member-id").map(String::from));
    id
}

fn get_linkedin_nick(url: Option<String>) -> Option<String> {
    println!("url withinj{:?}", url);
    match url {
        Some(url) => {
            // Split the URL by '/' and attempt to extract the username part
            let parts: Vec<&str> = url.split('/').collect();
            // The linkedin_nick should be the second to last element if the URL ends with '/'
            // or the last element if it doesn't.
            let linkedin_nick = parts.iter().rev().find(|&&part| !part.is_empty());
            linkedin_nick.map(|s| s.to_string())
        }
        None => None,
    }
}
fn get_title(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("div[class='text-body-medium break-words']");
    let title_selector = match title_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    let mut title = String::new();

    for element in document.select(&title_selector) {
        title = element.text().collect::<Vec<_>>().join(" ");
    }
    title = title.trim().to_string();
    Some(title)
}
fn get_name_tuple(html: &str) -> Option<(String, String)> {
    let fullname = scrap_full_name(&html);
    let clean_full_name = match fullname {
        Some(name) => remove_prefixes(&name),
        None => None,
    };
    let split_full_name = match clean_full_name {
        Some(name) => split_name(name),
        None => None,
    };
    split_full_name
}

fn scrap_full_name(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    // Define a selector for the LinkedIn URLs
    let name_selector =
        Selector::parse("h1[class='text-heading-xlarge inline t-24 v-align-middle break-words']");
    let name_selector = match name_selector {
        Ok(name) => name,
        Err(_) => return None,
    };
    let mut name: String = String::new();
    for element in document.select(&name_selector) {
        name = element.text().collect::<Vec<_>>().join(" ");
    }
    Some(name)
}

fn remove_prefixes(name: &str) -> Option<String> {
    let prefixes = [
        "Dr.",
        "Mr.",
        "Mrs.",
        "Ms.",
        "Prof.",
        "he/him/his",
        "He/Him/His",
        "Ph.D",
        "bsn",
        "ceo",
        "certified",
        "cpc",
        "dr.",
        "dr",
        "dr .",
        "drs",
        "drs .",
        "drs ",
        "expert",
        "freelance",
        "lion",
        "mba",
        "md",
        "md.",
        "msc",
        "ninja",
        "phd",
        "Ph.D.",
    ];
    let mut clean_name = String::from(name);
    for prefix in prefixes {
        if clean_name.starts_with(prefix) {
            clean_name = clean_name[prefix.len()..].trim().to_string();
            break;
        }
    }
    Some(clean_name)
}

fn split_name(name: String) -> Option<(String, String)> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() > 1 {
        let first_name = parts[0].to_string();
        let last_name = parts[1..].join(" ");
        Some((first_name, last_name))
    } else {
        Some((name, String::new())) // In case there's only one part, assume it's the first name
    }
}
