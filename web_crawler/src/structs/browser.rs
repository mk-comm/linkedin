use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BrowserInit {
    pub ip: String,
    pub username: String,
    pub password: String,
    pub user_agent: String,
    pub session_cookie: String,
    pub user_id: String,
    pub recruiter_session_cookie: Option<String>,
    pub bcookie: Option<String>,
    pub bscookie: Option<String>,
    pub fcookie: Option<String>,
    pub fidcookie: Option<String>,
    pub jsessionid: Option<String>,
}
