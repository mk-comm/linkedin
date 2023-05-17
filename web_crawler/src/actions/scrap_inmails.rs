use crate::structs::entry::Entry;
use crate::actions::start_browser::start_browser;
use scraper::{Html, Selector};

use crate::actions::wait::wait;
use crate::structs::inmail_conversation::InmailConversation;
use crate::structs::error::CustomError;


pub async fn scrap_inmails(entry: Entry) -> Result<(), CustomError> {

let api_key = entry.user_id.clone();
let browser = start_browser(entry).await?;

    // go to candidate page
browser
        .page
        .goto_builder("https://www.linkedin.com/talent/inbox/")
        .goto()
        .await?;
        
    wait(3, 15); // random delay
                 //check if connect button is present

let conversation_list = match browser.page.query_selector("div._conversations-container_zkxis6").await? {
      Some(conversation_list) => conversation_list,
      None => {
         wait(1, 5); // random delay
         browser.page.close(Some(false)).await?;
         browser.browser.close().await?; // close browser
         return Err(playwright::Error::ReceiverClosed.into());
      } // if search input is not found, means page was not loaded and sessuion cookie is not valid
   };




let document = Html::parse_document(conversation_list.inner_html().await?.as_str());
    let conversation_selector = Selector::parse("._card-container_z8knzq").unwrap();
    let name_selector = Selector::parse("._conversation-card-participant-name_z8knzq").unwrap();
    let url_selector = Selector::parse("._conversation-link_z8knzq").unwrap();
    let unread_selector = Selector::parse("._unread-badge_z8knzq").unwrap();
    let snippet_selector = Selector::parse("._conversation-snippet_z8knzq").unwrap();

    for conversation in document.select(&conversation_selector) {
        let id = conversation
            .select(&url_selector)
            .next()
            .map(|element| element.value().attr("id"))
            .unwrap_or(Some("Not found"));
        let name = conversation
            .select(&name_selector)
            .next()
            .map(|element| element.inner_html())
            .unwrap_or("Not found".to_string())
            .trim()
            .to_string();
        let url = conversation
            .select(&url_selector)
            .next()
            .map(|element| element.value().attr("href"))
            .unwrap_or(Some("Not found"));
         let unread = match conversation.select(&unread_selector).next() {
            Some(_) => true,
            None => false,
        };
        let _snippet = conversation
            .select(&snippet_selector)
            .next()
            .map(|element| element.inner_html())
            .unwrap_or("Not found".to_string());

         let conversation = InmailConversation::new(
            id.unwrap().to_string(),
            url.unwrap().to_string(),
            name,
            unread,
            api_key.clone(),
         );
         
         let messages_container = match browser.page.query_selector("div._messages-container_1j60am._divider_lvf5de").await? {
            Some(conversation_list) => conversation_list,
            None => {
               wait(1, 5); // random delay
               browser.page.close(Some(false)).await?;
               browser.browser.close().await?; // close browser
               return Err(playwright::Error::ReceiverClosed.into());
            } // if search input is not found, means page was not loaded and sessuion cookie is not valid
         };
         let html = messages_container.inner_html().await?;
         //println!("{:?}", conversation);
         scrap_message(conversation, html.as_str());
    }

wait(300, 700);












Ok(())
}

fn scrap_message(conversation: InmailConversation, html: &str) -> Result<(), playwright::Error> {
    

   let document = Html::parse_document(html);
   let message_id_selector = Selector::parse("._message-list-item_1gj1uc").unwrap();
   let sender_name_selector = Selector::parse("._headingText_e3b563").unwrap();
   let timestamp_selector = Selector::parse("time").unwrap();
   let message_text_selector = Selector::parse("._message-data-wrapper_1gj1uc div div div").unwrap();
   
   
   for message_element in document.select(&message_id_selector) {
      
      let message_id = message_element.value().id().unwrap_or_default();
       println!("Message ID: {}", message_id);
   
       if let Some(sender_element) = message_element.select(&sender_name_selector).next() {
           let sender_full_name = sender_element.inner_html();
           println!("Sender Full Name: {}", sender_full_name);
       }
   
       if let Some(timestamp_element) = message_element.select(&timestamp_selector).next() {
           let timestamp = timestamp_element.inner_html();
           println!("Timestamp: {}", timestamp);
       }

       if let Some(message_text_element) = message_element.select(&message_text_selector).next() {
           let message_text = message_text_element.inner_html();
           println!("Message Text: {}", message_text);
       }
   
       println!();
   }
    
    Ok(())
}