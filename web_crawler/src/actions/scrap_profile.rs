use crate::actions::start_browser::start_browser;
use playwright::api::Page;
use scraper::Selector;
use crate::structs::browser::BrowserInit;
use serde_json::json;
use crate::actions::wait::wait;
use crate::structs::candidate::Candidate;
use crate::structs::entry::Entry;
use crate::structs::error::CustomError;

pub async fn scrap_profile(entry: Entry) -> Result<(), CustomError> {
   let candidate = Candidate::new(
      entry.fullname.clone(),
      entry.linkedin.clone(),
      entry.message.clone(),
  );
  
  let browser_info = BrowserInit {
   ip: entry.ip,
   username: entry.username,
   password: entry.password,
   user_agent: entry.user_agent,
   session_cookie: entry.session_cookie,
   user_id: entry.user_id,
   recruiter_session_cookie: Some(entry.recruiter_session_cookie),
   };
   let browser = start_browser(browser_info).await?;

   let search_input = browser
        .page
        .query_selector("input[class=search-global-typeahead__input]")
        .await?;
   wait(3, 15); // random delay
                 //focus on search input and fill it with text
   match search_input {
        Some(search_input) => {
            search_input.hover_builder(); // hover on search input
            wait(1, 4); // random delay
            search_input.click_builder().click().await?; // click on search input
            wait(2, 5); // random delay
            search_input
                .fill_builder(&candidate.fullname)
                .fill()
                .await?; // fill search input with text
            wait(1, 5); // random delay
            search_input.press_builder("Enter").press().await?; // press Enter
            wait(2, 6); // random delay
        }
        None => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::SessionCookieExpired);
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };

    // go to candidate page
   browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await?
        .unwrap();
    wait(3, 15); // random delay

    let entity_urn = match find_entity_run(&browser.page).await {
      Ok(entity_urn) => entity_urn,
      Err(_) => {
          wait(1, 5); // random delay
          browser.page.close(Some(false)).await?;
          browser.browser.close().await?;
          return Err(playwright::Error::InitializationError.into());
      }
  };

  println!("entity_urn: {}", entity_urn);
  
   let contact_info = browser.page.query_selector("a#top-card-text-details-contact-info").await?.unwrap();
   let url = contact_info.get_attribute("href").await?;
   println!("url: {}", url.unwrap());



  let client = reqwest::Client::new();
    let payload = json!({
            "entity_urn": entity_urn,
            "linkedin": candidate.linkedin,
    });
    let _res = client
        .post("https://overview.tribe.xyz/api/1.1/wf/update_entity_urn")
        .json(&payload)
        .send()
        .await
        .unwrap();

   wait(5, 7);
   browser.page.close(Some(false)).await?;
   browser.browser.close().await?;

Ok(())
}

async fn find_entity_run(page: &Page) -> Result<String, playwright::Error> {

   // Find the target link
   let link_selector = Selector::parse("a").unwrap();
   let document = scraper::Html::parse_document(&page.content().await?);
   let mut entity_urn = String::new();

   for link in document.select(&link_selector) {
       let href = link.value().attr("href").unwrap_or_default();

       if href.contains("profileUrn=") {
           let parts: Vec<&str> = href.split("?profileUrn=urn%3Ali%3Afsd_profile%3A").collect();

           if parts.len() > 1 {
               entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();

               if entity_urn.is_empty() {
                   let parts: Vec<&str> = href.split("?profileUrn=urn%3Ali%3Afs_normalized_profile%3A").collect();
                   if parts.len() > 1 {
                       entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                   }
               }
           }

           if !entity_urn.is_empty() {
               break;
           }
       }
   }


   Ok(entity_urn)
}