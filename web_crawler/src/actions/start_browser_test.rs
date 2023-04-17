use playwright::Playwright;

use playwright::api::ProxySettings;

use crate::structs::browser::BrowserConfig;

#[allow(dead_code)]
pub async fn start_browser_test() -> Result<BrowserConfig, playwright::Error> {
    let proxy = ProxySettings {
        server: "44.155.71.29:8618".to_string(),
        username: Some("opncbnxd".to_string()),
        password: Some("v943hb1fn245".to_string()),
        bypass: None,
    };

    let playwright = Playwright::initialize().await?;

    playwright.prepare()?; // Install browsers

    let chromium = playwright.chromium();

    let browser = chromium
        .launcher()
        .proxy(proxy)
        .headless(false)
        .launch()
        .await?;

    let context = browser.context_builder().build().await?;

    let page = context.new_page().await?;

    let build = page
        .goto_builder("file:///home/mikhail/Documents/(33)%20Messaging%20_%20LinkedIn.html")
        .goto()
        .await;

    match &build {
        Ok(_) => print!("ok"),
        Err(e) => print!("err4, {}", e),
    } // if error when proxy is not working

    let browser_config = BrowserConfig {
        proxy: None,
        playwright: playwright,
        browser_type: chromium,
        browser: browser,
        context: context,
        page: page,
        build: build.unwrap().unwrap(),
    };
    return Ok(browser_config);
}
