#[derive(Debug)]
pub struct Message {
    pub message_text: String,
    pub sender: String,
    pub timestamp: String,
    pub url_send_from: String,
    pub url_send_to: String,
    pub received: bool,
    //pub conversation_with: String,
    //pub connection_level: String,
}
#[allow(dead_code)]
impl Message {
    pub fn new(
        message_text: String,
        sender: String,
        timestamp: String,
        url_send_from: String,
        url_send_to: String,
        received: bool,
    ) -> Self {
        Message {
            message_text,
            sender,
            timestamp,
            url_send_from,
            url_send_to,
            received,
            //conversation_with,
        }
    }
}
