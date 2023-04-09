pub struct User {
    pub user_agent: String,
    pub session_cookie: String,
    pub user_id: String,
}

impl User {
    pub fn new(user_agent: String, session_cookie: String, user_id: String) -> Self {
        User {
            user_agent,
            session_cookie,
            user_id,
        }
    }
}
