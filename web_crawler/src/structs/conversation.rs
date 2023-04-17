pub struct Conversation {
    pub id: String,
    pub thread_url: String,
    pub candidate_name: String,
    pub timestamp: String,
    pub unread: bool,
    pub api_key: String,
}
#[allow(dead_code)]
impl Conversation {
    pub fn new(
        id: String,
        thread_url: String,
        candidate_name: String,
        timestamp: String,
        unread: bool,
        api_key: String,
    ) -> Self {
        Conversation {
            id,
            thread_url,
            candidate_name,
            timestamp,
            unread,
            api_key,
        }
    }
}
