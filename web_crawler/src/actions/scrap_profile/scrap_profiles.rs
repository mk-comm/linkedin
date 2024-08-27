use crate::actions::scrap_profile::scrap_each_profile::scrap_each_profile;
use crate::actions::scrap_profile::scrap_each_profile::send_search_status;
use crate::actions::start_browser_new::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserConfig;
use crate::structs::browser::BrowserInit;
use crate::structs::entry::EntryScrapProfile;
use crate::structs::entry::Url;
use crate::structs::error::CustomError;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task;

pub async fn scrap_profile(entry: EntryScrapProfile) -> Result<(), CustomError> {
    //let user_id = Arc::new(RwLock::new(entry.user_id.clone()));

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
        headless: true,
        session_cookie: entry.cookies.session_cookie,
        recruiter_session_cookie: entry.cookies.recruiter_session_cookie,
        bscookie: entry.cookies.bscookie,
        bcookie: entry.cookies.bcookie,
        fcookie: entry.cookies.fcookie,
        fidcookie: entry.cookies.fidcookie,
        jsessionid: entry.cookies.jsessionid,
    };

    let browser = start_browser(browser_info.clone()).await?;
    let browser = Arc::new(RwLock::new(browser));
    let mut tasks = Vec::new();
    send_search_status("Connected to linkedin", &aisearch, batch.as_str(), "none").await?;

    run_loop(
        urls,
        browser,
        aisearch.clone(),
        job.clone(),
        sourcer.clone(),
        search_url.clone(),
        &mut tasks,
    )
    .await;

    let results = join_all(tasks).await;

    // Check results for any errors
    for result in results {
        println!("{:?}", result);
        match result {
            Ok(nested_result) => {
                if nested_result.is_err() {
                    return Err(CustomError::ButtonNotFound(format!(
                        "Nestef Error {:?} /scrap_profile",
                        nested_result
                    )));
                }
            }
            Err(e) => {
                return Err(CustomError::ButtonNotFound(format!(
                    "Error {} /scrap_profile",
                    e
                )));
            }
        };
    }
    Ok(())
}

async fn run_loop(
    urls: Vec<Url>,
    browser: Arc<RwLock<BrowserConfig>>,
    aisearch: Option<String>,
    job: Option<String>,
    sourcer: Option<String>,
    search_url: Option<String>,
    tasks: &mut Vec<task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
) {
    for url in urls {
        let aisearch = Arc::new(RwLock::new(aisearch.clone()));
        let job = Arc::new(RwLock::new(job.clone()));
        let sourcer = Arc::new(RwLock::new(sourcer.clone()));
        let search_url = Arc::new(RwLock::new(search_url.clone()));
        let url_one = Arc::new(RwLock::new(url.clone()));

        let browser_clone = Arc::clone(&browser);
        let aisearch_clone = Arc::clone(&aisearch);
        let job_clone = Arc::clone(&job);
        let sourcer_clone = Arc::clone(&sourcer);
        let url_clone = Arc::clone(&url_one);
        let search_url_clone = Arc::clone(&search_url);

        let task = task::spawn(async move {
            let browser = browser_clone;
            let aisearch = aisearch_clone;
            let job = job_clone;
            let sourcer = sourcer_clone;

            scrap_each_profile(browser, url_clone, job, sourcer, aisearch, search_url_clone)
                .await?;

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

        wait(20, 21);
        tasks.push(task);
    }
}
