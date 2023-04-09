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
