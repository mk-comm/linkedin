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

#[derive(Debug, Deserialize, Serialize)]
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
    "recruiter_session_cookie":"AQJ2PTEmY2FwX3NlYXQ9MjU2MjQ0MTEzJmNhcF9hZG1pbj1mYWxzZSZjYXBfa249MjQxNDY4ODAzFT9osKrQvUG0aR2ol-xlpTVVMvI"
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