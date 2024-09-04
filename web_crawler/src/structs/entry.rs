use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[warn(dead_code)]
pub struct Entry {
    pub message_id: String,
    pub webhook: String,
    pub fullname: String,
    pub linkedin: String,
    pub message: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub user_id: String,
    pub entity_urn: String,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
    pub regular: bool,
    pub recruiter: bool,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRegular {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub regular: bool,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRecruiter {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub recruiter: bool,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapProjects {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub target_url: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryAddProfilesToProjects {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub target_url: String,
    pub candidates: Vec<CandidateUrl>,
    pub cookies: Cookies,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CandidateUrl {
    pub url: String,
    pub project: String,
    pub stage: String,
    pub id: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EntrySendInmail {
    pub message_id: String,
    pub webhook: String,
    pub fullname: String,
    pub linkedin: String,
    pub message: String,
    pub subject: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub user_id: String,
    pub file_url: String,
    pub file_name: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapProfile {
    pub webhook: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub user_id: String,
    pub sourcer: String,
    pub job: String,
    pub aisearch: String,
    pub urls: Vec<Url>,
    pub batch_id: String,
    pub search_url: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cookies {
    pub session_cookie: String,
    pub recruiter_session_cookie: Option<String>,
    pub bcookie: Option<String>,
    pub bscookie: Option<String>,
    pub fcookie: Option<String>,
    pub fidcookie: Option<String>,
    pub jsessionid: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Url {
    pub url: String,
    pub batch_id: String,
    pub url_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntrySendConnection {
    pub message_id: String,
    pub webhook: String,
    pub fullname: String,
    pub linkedin: String,
    pub message: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub user_id: String,
    pub cookies: Cookies,
    pub check_reply: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapConnection {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapSearchRegular {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub url: String,
    pub result_url: String,
    pub url_list_id: String,
    pub aisearch: String,
    pub cookies: Cookies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapSearchRecruiter {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub url: String,
    pub result_url: String,
    pub url_list_id: String,
    pub aisearch: String,
    pub cookies: Cookies,
    pub urls_scraped: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PhantomGetJson {
    pub body: Vec<PhantomJsonProfile>,
    pub sourcer: Option<String>,
    pub job: Option<String>,
    pub search_url: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhantomGeneral {
    pub imgUrl: Option<String>,
    pub fullName: Option<String>,
    pub headline: Option<String>,
    pub location: Option<String>,
    pub profileUrl: Option<String>,
    pub connectionDegree: Option<String>,
    pub description: Option<String>,
    pub firstName: Option<String>,
    pub lastName: Option<String>,
    pub userId: Option<String>,
    pub vmid: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct PhantomJsonProfile {
    pub general: PhantomGeneral,
    pub jobs: Option<Vec<PhantomJobs>>,
    pub schools: Option<Vec<PhantomSchools>>,
    pub timestamp: Option<String>,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PhantomSchools {
    pub schoolUrl: Option<String>,
    pub schoolName: Option<String>,
    pub logoUrl: Option<String>,
    pub degree: Option<String>,
    pub dateRange: Option<String>,
    pub description: Option<String>,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PhantomJobs {
    pub companyUrl: Option<String>,
    pub companyName: Option<String>,
    pub logoUrl: Option<String>,
    pub jobTitle: Option<String>,
    pub dateRange: Option<String>,
    pub duration: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
}
