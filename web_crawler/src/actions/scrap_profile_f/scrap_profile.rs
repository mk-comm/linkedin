use crate::actions::scrap_profile_f::misc::serialize_option_string;
use crate::actions::scrap_profile_f::scrap_education::parse_education;
use crate::actions::scrap_profile_f::scrap_education::Education;
use crate::actions::scrap_profile_f::scrap_experience_new_tab::{parse_experience, Experience};
use crate::actions::scrap_profile_f::scrap_experience_same_tab::parse_experience_same_page;
use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use playwright::api::Page;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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

pub async fn scrap_profile(entry: EntrySendConnection) -> Result<(), CustomError> {
    //path to  local browser
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );
    let browser_info = BrowserInit {
        ip: entry.ip,
        username: entry.username,
        password: entry.password,
        user_agent: entry.user_agent,
        session_cookie: entry.session_cookie,
        user_id: entry.user_id,
        recruiter_session_cookie: None,
        headless: false,
    };
    let browser = start_browser(browser_info).await?;

    let search_input = browser
        .page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;
    wait(3, 15); // random delay
                 //focus on search input and fill it with text
    match search_input {
        Some(search_input) => {
            search_input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            search_input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            search_input
                .fill_builder(&candidate.fullname)
                .fill()
                .await?; // fill search input with text
            wait(1, 5); // random delay
            search_input.press_builder("Enter").press().await?; // press Enter
            wait(2, 6); // random delay
        }
        None => {
            wait(1, 5); // random delay
        } //
    };
    // go to candidate page
    let go_to = browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await;
    let mut x = 0;
    if go_to.is_err() {
        while x <= 3 {
            wait(3, 6);
            let build = browser
                .page
                .goto_builder(candidate.linkedin.as_str())
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
            "body[class='render-mode-BIGPIPE nav-v2 ember-application boot-complete icons-loaded']",
        )
        .await?;
    let html = match html_body {
        Some(body) => body.inner_html().await?,
        None => {
            wait(1, 5);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(CustomError::ButtonNotFound("Body is not found".to_string()));
        }
    };
    //wait(10000, 10000);
    let linkedin_url = get_linkedin_url(browser.page.clone());
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
    let current_company_name: &str;
    let current_company_id: &str;
    let profile_picture = get_profile_picture(&html);
    let about = get_about(&html);
    let education: &str;
    let experience: &str;
    let certifications: &str;
    let languages: &str;
    let location = get_location(&html);
    let entity_urn = find_entity_urn(&html);
    println!("url {:?}", &linkedin_url.clone());
    println!("fullname {:?}", full_name);
    println!("first {:?}", first_name);
    println!("last {:?}", last_name);
    println!("tittle {:?}", current_tittle);
    println!("nick {:?}", linkedin_nick);
    println!("id {:?}", linkedin_main_id);
    println!("profice pic {:?}", profile_picture);
    println!("about {:?}", about);
    println!("location {:?}", location);
    println!("entity {:?}", entity_urn);

    let exp_vec = parse_experience_same_page(browser.page).await?;
    for exp in exp_vec {
        println!("exp {:?}", exp)
    }
    /*
        let experience_url = format!("{}/details/experience", candidate.linkedin.as_str());
        let go_to = browser
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
                return Err(CustomError::ButtonNotFound("Body is not found".to_string()));
            }
        };
        let experience = parse_experience(&html);

        let education_url = format!("{}/details/education", candidate.linkedin.as_str());
        let go_to = browser
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
                return Err(CustomError::ButtonNotFound("Body is not found".to_string()));
            }
        };
        let education = parse_education(&html);
        for exp in education {
            println!("{:?}", exp)
        }
    */
    Ok(())
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
    println!("href{}", href);
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

fn get_profile_picture(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let picture_selector =
        Selector::parse("img[class='pv-top-card-profile-picture__image pv-top-card-profile-picture__image--show evi-image ember-view']");
    let picture_selector = match picture_selector {
        Ok(selector) => selector,
        Err(_) => return None,
    };
    let picture = document
        .select(&picture_selector)
        .next()
        // Extract the 'src' attribute if it exists
        .and_then(|element| element.value().attr("src").map(String::from));
    picture
}
fn get_linkedin_main_id(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let id_selector =
        Selector::parse("section[class='artdeco-card KjYJHUsAbArdmYARDTlIvPWCMLPtSGIXXQFs']");
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
