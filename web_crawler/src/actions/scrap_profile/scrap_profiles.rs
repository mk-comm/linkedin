

use crate::actions::init_browser::init_browser;
use crate::actions::scrap_profile::scrap_each_profile::scrap_each_profile_main;
use crate::actions::scrap_profile::scrap_each_profile::send_search_status;
use crate::actions::scrap_profile::scrap_each_profile::Profile;
use crate::actions::scrap_profile::scrap_experience_new_tab::get_experience;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::{EntryScrapProfile, Url};
use crate::structs::error::CustomError;
use base64::encode;
use serde_json::json;
use tracing::{error, info};

use thirtyfour::{WebDriver, WindowHandle};

pub async fn scrap_profile(entry: EntryScrapProfile) -> Result<(), CustomError> {
    let job = Some(entry.job.clone()).filter(|j| !j.is_empty());
    let aisearch = Some(entry.aisearch.clone()).filter(|s| !s.is_empty());
    let sourcer = Some(entry.sourcer.clone()).filter(|s| !s.is_empty());
    let search_url = Some(entry.search_url.clone()).filter(|s| !s.is_empty());
    let urls = entry.urls.clone();
    let batch = entry.batch_id.clone();

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
    send_search_status("Connected to linkedin", &aisearch, batch.as_str(), "none").await?;
    let chunks = urls.chunks(5);

    for chunk in chunks {
        let urls = chunk.to_vec();
        let result = start_scraping(&browser, aisearch.clone(), &batch, urls, job.clone(), sourcer.clone(), search_url.clone()).await;
        if let Err(error) = result {
            browser.quit().await?;
            return Err(error);
        };

    }
    

    Ok(())
}

async fn start_scraping(
    browser: &WebDriver,
    aisearch: Option<String>,
    batch: &str,
    urls: Vec<Url>,
    job: Option<String>,
    sourcer: Option<String>,
    search_url: Option<String>,
) -> Result<(), CustomError> {
    send_search_status("Connected to linkedin", &aisearch, batch, "none").await?;

    let tabs = open_urls(&browser, urls).await?;
    let main = scrap_main_profiles(&browser, tabs, job, sourcer, aisearch, search_url).await?;
    let _profiles = scrap_experience_to_profile(&browser, main).await?;
    Ok(())
}

async fn open_urls(
    browser: &WebDriver,
    urls: Vec<Url>,
) -> Result<Vec<(WindowHandle, Url)>, CustomError> {
    let mut tabs = vec![];
    for url in urls {
        let url_copy = url.clone();
        let window = browser.new_tab().await?;
        browser.switch_to_window(window.clone()).await?;
        browser.goto(url_copy.url).await?;
        tabs.push((window, url));
    }
    Ok(tabs)
}

async fn scrap_main_profiles(
    browser: &WebDriver,
    tabs: Vec<(WindowHandle, Url)>,
    job: Option<String>,
    sourcer: Option<String>,
    ai_search: Option<String>,
    search_url: Option<String>,
) -> Result<Vec<Profile>, CustomError> {
    let mut profiles: Vec<Profile> = Vec::new();
    for tab in tabs {
        browser.switch_to_window(tab.0).await?;
        let url_id = tab.1;
        let profile = scrap_each_profile_main(
            browser,
            job.clone(),
            sourcer.clone(),
            ai_search.clone(),
            search_url.clone(),
            url_id.url_id,
        )
        .await?;
        profiles.push(profile);
    }
    Ok(profiles)
}

async fn scrap_experience_to_profile(
    browser: &WebDriver,
    profiles: Vec<Profile>,
) -> Result<Vec<Profile>, CustomError> {
    let mut profiles_new = Vec::new();
    for profile in profiles {
        let tab = browser.new_tab().await?;
        browser.switch_to_window(tab).await?;
        let experience =
            get_experience(&browser, &profile.linkedin.clone().unwrap().clone()).await?;
        let company = if experience.len() > 0 {
            experience[0].companyName.clone()
        } else {
            None
        };
        let company_unique = if experience.len() > 0 {
            experience[0].companyId.clone()
        } else {
            None
        };
        let profile_new = Profile {
            experience,
            company,
            company_unique,

            ..profile
        };
        send_url_chromedata_viewed(&profile_new).await?;
        send_url_update(&profile_new.profile_url_id, &profile_new.linkedin, &profile_new).await?;
        profiles_new.push(profile_new);
    }
    Ok(profiles_new)
}
#[allow(deprecated)]
async fn send_url_chromedata_viewed(profile: &Profile) -> Result<(), CustomError> {
    let serialized = serde_json::to_vec(&profile).unwrap();
    let encoded = encode(&serialized);
    const WEBHOOK_URL: &str = "https://overview.tribe.xyz/api/1.1/wf/chromedata_view";
    //const WEBHOOK_URL: &str = "https://webhook.site/c58568dc-6357-4aa4-96c2-79d6f22c1ede";
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
    linkedin_url: &Option<String>, profile: &Profile
) -> Result<(), reqwest::Error> {
    let serialized = serde_json::to_vec(&profile).unwrap();
    let encoded = encode(&serialized);

    let max_retries = 5;
    let client = reqwest::Client::new();
    let urls_json = json!({
        "url_id": url_id,
        "linkedin": linkedin_url,
        "b64": encoded
    });
    let target_url = "https://overview.tribe.xyz/api/1.1/wf/tribe_scrap_search_update_url";

    //let target_url = "https://webhook.site/edf0826d-61e4-4de5-bdd1-678d485785a9";
    let mut retries = 0;
    loop {
        let response = client.post(target_url).json(&urls_json).send().await;
        match response {
            Ok(res) => {
                info!(
                    "Send_urls/url_update/Ok: {}, status: {}",
                    url_id,
                    res.status()
                );
                return Ok(());
            }
            Err(error) => {
                if retries < max_retries {
                    retries += 1;
                    wait(1, 1);
                    continue;
                } else {
                    error!(error = ?error, "Send_urls/url_update/Error {} returned error {}", url_id, error);
                    return Err(error);
                }
            }
        }
    }
}
