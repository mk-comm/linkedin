#[derive(Debug, Clone)]
pub struct InmailConversation {
    pub id: String,
    pub thread_url: String,
    pub candidate_name: String,
    pub unread: bool,
    pub api_key: String,
}
#[allow(dead_code)]
impl InmailConversation {
    pub fn new(
        id: String,
        thread_url: String,
        candidate_name: String,
        unread: bool,
        api_key: String,
    ) -> Self {
        InmailConversation {
            id,
            thread_url,
            candidate_name,
            unread,
            api_key,
        }
    }
}
