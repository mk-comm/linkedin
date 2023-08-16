use playwright::api::{
    Browser, BrowserContext, BrowserType, Page, Playwright, ProxySettings, Response,
};

pub struct BrowserConfig {
    pub proxy: Option<ProxySettings>,
    pub playwright: Playwright,
    pub browser_type: BrowserType,
    pub browser: Browser,
    pub context: BrowserContext,
    pub page: Page,
    pub build: Response,
}

#[derive(Debug)]
pub struct BrowserInit {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub session_cookie: String,
    pub user_id: String,
    pub recruiter_session_cookie: Option<String>,
}
