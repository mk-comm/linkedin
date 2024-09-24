use crate::actions::add_to_project::change_stage_project::change_stage;

use crate::actions::add_to_project::save_each_to_project::save_each;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::CandidateUrl;
use crate::structs::entry::EntryAddProfilesToProjects;
use crate::structs::error::CustomError;
use serde_json::json;
use thirtyfour::{By, WebDriver};
use tracing::{error, info};

use crate::actions::init_browser::{init_browser, send_screenshot};

pub async fn save(entry: EntryAddProfilesToProjects) -> Result<(), CustomError> {
    let target_url = entry.target_url.clone();
    let user_id = entry.user_id.clone();
    let candidates = entry.candidates.clone();
    let browser = init(entry).await?;
    let result = init_save(candidates, &browser, &target_url).await;
    match result {
        Ok(text) => {
            let screenshot = browser.screenshot_as_png().await?;
            send_screenshot(
                screenshot,
                &user_id,
                "Candidate added to the project",
                &user_id,
                "Save to project",
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
                error.to_string().as_str(),
                &user_id,
                "Save to project",
            )
            .await?;
            return Err(error);
        }
    }
}

pub async fn init_save(
    candidates: Vec<CandidateUrl>,
    browser: &WebDriver,
    target_url: &str,
) -> Result<(), CustomError> {
    for candidate in candidates {
        let candidate_linkedin = candidate.url;
        save_each(&browser, &candidate_linkedin, candidate.project.as_str()).await?;
        wait(3, 5);
        if candidate.stage != "1. uncontacted" {
            change_stage(&browser, candidate.project.as_str()).await?;
        }

        send_urls(&target_url, candidate.id.as_str()).await?;
        wait(7, 10); // random delay
    }
    Ok(())
}

async fn init(entry: EntryAddProfilesToProjects) -> Result<WebDriver, CustomError> {
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

async fn send_urls(target_url: &str, candidate_id: &str) -> Result<(), CustomError> {
    let max_retries = 5;
    let client = reqwest::Client::new();

    let urls_json = json!({
        "candidate_id": candidate_id,
    });

    let mut retries = 0;
    loop {
        let response = client.post(target_url).json(&urls_json).send().await;
        match response {
            Ok(res) => {
                if res.status() == 200 {
                    info!(
                        "Send_urls/AddProfilesToProjects/Ok: {}, status: {}/URL {}",
                        candidate_id,
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
                            "Send_urls/AddProfilesToProjects/Error {}: status {}/URL: {}",
                            candidate_id,
                            res.status(),
                            target_url
                        );
                        return Err(CustomError::ButtonNotFound(
                            "Send url status is not 200, Status/AddProfilesToProjects".to_string(),
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
                    error!(error = ?error, "Send_urls/AddProfilesToProjects/Error {} returned error {}/URL: {}", candidate_id, error, target_url);

                    return Err(CustomError::ButtonNotFound(
                        "AddProfilesToProjects, Error".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}
