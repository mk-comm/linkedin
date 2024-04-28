use crate::actions::scrap_profile_f::misc::serialize_option_string;
use crate::actions::scrap_profile_f::scrap_education::parse_education;
use crate::actions::scrap_profile_f::scrap_education::Education;
use crate::actions::scrap_profile_f::scrap_experience_new_tab::{parse_experience, Experience};
use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserConfig;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapProfile;
use crate::structs::error::CustomError;
use playwright::api::Page;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use serde_json::json;
#[allow(deprecated)]
use base64::encode;
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
struct BodyJson {
    body: Profile,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Profile {
    AI: bool,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    first: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    last: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    email: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    job: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    sourcer: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    title: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin_unique_number: Option<String>,
    connectionLevel: Option<i32>,
    #[serde(serialize_with = "serialize_option_string")]
    company: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    company_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    about: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    profilePicture: Option<String>,
    education: Vec<Education>,
    experience: Vec<Experience>,
    #[serde(serialize_with = "serialize_option_string")]
    viewedIn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    location: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    entityUrn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    extension_version: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    timestamp: Option<String>,
    search_url: Option<String>,
}

pub async fn scrap_profile(entry: EntryScrapProfile) -> Result<(), CustomError> {
    
    let job = if entry.job.is_empty() {
        None
    } else {
        Some(entry.job.clone())
            
    };
    let aisearch = if entry.aisearch.is_empty() {
        None
    } else {
        Some(entry.aisearch.clone())
            
    };

    let sourcer = if entry.sourcer.is_empty() {
        None
    } else {
        Some(entry.sourcer.clone())
            
    };
    let urls = entry.urls;

    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: None,
        headless: true,
    };
    let browser = start_browser(browser_info).await?;

    for url in urls {
        scrap_each_profile(&browser, &url.url, &job, &sourcer, &aisearch, &url.url_id).await?;
    }

    Ok(())
}
async fn scrap_each_profile(
    browser: &BrowserConfig,
    url: &str,
    job: &Option<String>,
    sourcer: &Option<String>,
    aisearch: &Option<String>,
    url_id: &str,
) -> Result<(), CustomError> {

let job = job;
    let search_url = aisearch;
    let sourcer = sourcer;
    // go to candidate page
    let go_to = browser
        .page
        .goto_builder(url)
        .goto()
        .await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser
                .page
                .goto_builder(url)
                .goto()
                .await;
            if build.is_ok() {
                break;
            } else if build.is_err() && x == 3 {
                wait(3, 6);
                browser.page.close(Some(false)).await?;
                browser.browser.close().await?; // close browser
                return Err(CustomError::ButtonNotFound(
                    "Candidate page is not loading/Connection".to_string(),
                )); // if error means page is not loading
            }
            x += 1;
            //println!("retrying to load page")
        }
        wait(1, 3);
    }

    wait(7, 21); // random delay
                 //check if connect button is present

    let page_not_found = browser
        .page
        .query_selector("header[class='not-found__header not-found__container']")
        .await?;
    if page_not_found.is_some() {
        wait(1, 5);
        browser.page.close(Some(false)).await?;
        browser.browser.close().await?;
        return Err(CustomError::ButtonNotFound(
            "Page does not exist".to_string(),
        ));
    }

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
    //wait(10000, 10000);
    let linkedin_url = get_linkedin_url(browser.page.clone());
    let linkedin_url_update = linkedin_url.clone();
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
    let linkedin_nick = get_linkedin_nick(linkedin_url.clone());
    let linkedin_main_id = get_linkedin_main_id(&html);
    let profile_picture = get_profile_picture(&html);
    let about = get_about(&html);
    let connection_level = Some(5);
    let location = get_location(&html);
    let entity_urn = find_entity_urn(&html);

    let experience_url = format!("{}/details/experience", url);
    let _go_to = browser
        .page
        .clone()
        .goto_builder(experience_url.as_str())
        .goto()
        .await?;

    wait(5, 13);
    let html_body = browser
        .page
        .query_selector("div[class='authentication-outlet']")
        .await?;
    let html = match html_body {
        Some(body) => body.inner_html().await?,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "1Body is not found".to_string(),
            ));
        }
    };
    let experience = parse_experience(&html);
    let current_company_name = &experience[0].companyName;
    let current_company_id = &experience[0].companyId;

    let education_url = format!("{}/details/education", url);
    let _go_to = browser
        .page
        .clone()
        .goto_builder(education_url.as_str())
        .goto()
        .await?;

    wait(5, 13);
    let html_body = browser
        .page
        .query_selector("main[class='scaffold-layout__main']")
        .await?;
    let html = match html_body {
        Some(body) => body.inner_html().await?,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound(
                "2Body is not found".to_string(),
            ));
        }
    };
    let education = parse_education(&html);

    let profile = Profile {
        AI: true,
        linkedin: linkedin_url,
        first: first_name,
        last: last_name,
        email: None,
        job: job.clone(),
        sourcer: sourcer.clone(),
        title: current_tittle,
        linkedin_unique: linkedin_nick,
        linkedin_unique_number: linkedin_main_id,
        connectionLevel: connection_level,
        company: current_company_name.clone(),
        company_unique: current_company_id.clone(),
        about,
        profilePicture: profile_picture,
        education,
        experience,
        viewedIn: None,
        location,
        entityUrn: entity_urn,
        extension_version: None,
        timestamp: None,
        search_url: search_url.clone()
    };

    send_url_chromedata_viewed(profile).await?;
    send_url_update(url_id, linkedin_url_update).await?;

    Ok(())
}

#[allow(deprecated)]
async fn send_url_chromedata_viewed(profile: Profile) -> Result<(), CustomError> {
    let serialized = serde_json::to_vec(&profile).unwrap();
    let encoded = encode(&serialized);
    const WEBHOOK_URL: &str = "https://webhook-test.com/c1d9b28b2cb8c5f2c14bacf71c841210";
    let client = reqwest::Client::new();

    let target_json = json!({ 
        "b64": encoded });
    let res = client.post(WEBHOOK_URL).json(&target_json).send().await;
    match res {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    Ok(())
}

async fn send_url_update(
    url_id: &str,
    linkedin_url: Option<String>
) -> Result<(), reqwest::Error> {
    let max_retries = 5;
     let client = reqwest::Client::new();
    let urls_json = json!({ 
        "url_id": url_id,
    "linkedin": linkedin_url
    });
    let target_url = "https://overview.tribe.xyz/version-test/api/1.1/wf/tribe_scrap_search_update_url";
    
    let mut retries = 0;
    loop {
        let response = client.post(target_url).json(&urls_json).send().await;
        match response {
            Ok(res) => {
                info!("Send_urls/url_update/Ok: {}, status: {}", url_id, res.status());
                return Ok(());
            },
            Err(error) => {
                if retries < max_retries {
                    retries += 1;
                    wait(1,1);
                    continue;
                } else {
                    error!(error = ?error, "Send_urls/url_update/Error {} returned error {}", url_id, error);
                    return Err(error);
                }
            }
        }
    }
}

fn find_entity_urn(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a").unwrap();

    // Iterate over all <a> tags to find the URN
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            if let Some(urn) = extract_urn_from_href(href) {
                return Some(urn.replace("urn%3Ali%3Afsd_profile%3A", ""));
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

fn get_linkedin_url(page: Page) -> Option<String> {
    let page_url = page.url();
    match page_url {
        Ok(url) => return Some(url),
        Err(_) => return None,
    }
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
