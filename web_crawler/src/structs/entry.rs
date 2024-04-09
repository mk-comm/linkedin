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
    pub session_cookie: String,
    pub user_id: String,
    pub entity_urn: String,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
    pub recruiter_session_cookie: String,
    pub regular: bool,
    pub recruiter: bool,
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
    pub session_cookie: String,
}

/*
{
    "ip": "154.4.174.216:5866",
    "username": "username",
    "password": "password",
    "webhook": "webhook",
    "user_id": "user_id",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "session_cookie",
    "regular": false
}
*/

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryRecruiter {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub recruiter: bool,
    pub session_cookie: String,
    pub recruiter_session_cookie: String,
    pub recruiter_stage_interested: String,
    pub recruiter_stage_not_interested: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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
    pub recruiter_session_cookie: String,
    pub session_cookie: String,
    pub file_url: String,
    pub file_name: String,
}

/* EntrySendInmail
{
    "message_id": "some",
    "webhook": "webhook",
    "fullname": "some",
    "linkedin": "some",
    "message": "some",
    "subject": "some",
    "ip": "104.239.18.22:5426",
    "username": "username",
    "password": "password",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a",
    "user_id": "qePDD2xm7Sap0AgUqexqbUx9WmzfOhoZYhWzcnZHlmM",
    "recruiter_session_cookie":"AQJ2PTEmY2FwX3NlYXQ9MjU2MjQ0MTEzJmNhcF9hZG1pbj1mYWxzZSZjYXBfa249MjQxNDY4ODAzFT9osKrQvUG0aR2ol-xlpTVVMvI",
    "file_url": "https://www.file.com",
    "file_name": "file.pdf",

}

*/

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
    pub session_cookie: String,
}

/* EntrySendConnection
{
    "message_id": "some",
    "webhook": "webhook",
    "fullname": "some",
    "linkedin": "some",
    "message": "some",
    "ip": "104.239.18.22:5426",
    "username": "username",
    "password": "password",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a",
    "user_id": "qePDD2xm7Sap0AgUqexqbUx9WmzfOhoZYhWzcnZHlmM"
}

*/
#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapConnection {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub session_cookie: String,
}

/* EntryScrapConnection
{
    "webhook": "webhook",
    "ip": "104.239.18.22:5426",
    "username": "username",
    "password": "password",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a",
    "user_id": "qePDD2xm7Sap0AgUqexqbUx9WmzfOhoZYhWzcnZHlmM"
}

*/

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapSearchRegular {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub session_cookie: String,
    pub url: String,
    pub result_url: String,
    pub url_list_id: String,
    pub aisearch: String,
}

/* EntryScrapSearchRegular
{
    "webhook": "webhook",
    "ip": "104.239.18.22:5426",
    "username": "username",
    "password": "password",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a",
    "user_id": "qePDD2xm7Sap0AgUqexqbUx9WmzfOhoZYhWzcnZHlmM",
    "url": "https://www.linkedin.com/search/results/people/?currentCompany=%5B%221441",
    "result_url" : "https://www.result.com",
    "url_list_id" : "id78",
    "aisearch": "uniqueid"
}

*/

#[derive(Debug, Deserialize, Serialize)]
pub struct EntryScrapSearchRecruiter {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub webhook: String,
    pub user_id: String,
    pub session_cookie: String,
    pub recruiter_session_cookie: String,
    pub url: String,
    pub result_url: String,
    pub url_list_id: String,
    pub aisearch: String,
}

/* EntryScrapSearchRecruiter
{
    "webhook": "webhook",
    "ip": "104.239.18.22:5426",
    "username": "username",
    "password": "password",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36",
    "session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a",
    "recruiter_session_cookie": "-cJKYmaFKFjZYydPcD0wV0tKxUBR1U9nTFASLc9a"
    "user_id": "qePDD2xm7Sap0AgUqexgtqbUx9WmzfOhoZYhWzcnZHlmM",
    "url": "https://www.linkedin.com/search/results/people/?currentCompany=%5B%221441",
    "result_url" : "https://www.result.com",
    "url_list_id" : "id78",
    "aisearch": "uniqueid"
}
*/

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
