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
}

impl Entry {
    #[allow(dead_code)]
    pub fn new(
        message_id: String,
        webhook: String,
        fullname: String,
        linkedin: String,
        message: String,
        ip: String,
        username: String,
        password: String,
        user_agent: String,
        session_cookie: String,
        user_id: String,
    ) -> Self {
        Entry {
            message_id,
            webhook,
            fullname,
            linkedin,
            message,
            ip,
            username,
            password,
            user_agent,
            session_cookie,
            user_id,
        }
    }
}
