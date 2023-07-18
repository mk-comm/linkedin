/* 
1. create a json format for inmails 
2. Start inmail browser
   check proxy
   check cookies
2. send inmails

*/
use playwright::api::Page;
use crate::structs::browser::BrowserInit;
use scraper::Selector;
use crate::actions::start_browser::start_browser;
use crate::structs::entry::EntrySendInmail;
use crate::structs::error::CustomError;
use crate::actions::wait::wait;
use crate::structs::candidate::Candidate;

pub async fn send_inmails(entry: EntrySendInmail) -> Result<(), CustomError> {
    let candidate = Candidate::new(
        entry.fullname.clone(),
        entry.linkedin.clone(),
        entry.message.clone(),
    );

    let subject = entry.subject.clone();

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
        } // if search input is not found, means page was not loaded and sessuion cookie is not valid
    };


    // go to candidate page
    let mut go_to = browser
        .page
        .goto_builder(candidate.linkedin.as_str())
        .goto()
        .await;
    let mut x = 0;
        if go_to.is_err() {
        
            while x <= 3 {
                wait(3, 6);
                let build = browser
                .page
                .goto_builder(candidate.linkedin.as_str())
                .goto()
                .await;
                if build.is_ok() {
                    go_to = build;
                    break;
                } else if build.is_err() && x == 3 {
                    wait(3, 6);
                    browser.page.close(Some(false)).await?;
                    browser.browser.close().await?; // close browser
                    return Err(CustomError::ButtonNotFound("Candidate page is not loading/Inmail_regular".to_string())); // if error means page is not loading
                }
                x += 1;
                println!("retrying to load page")
            }
            wait(1, 3);
        }


    wait(3, 15); // random delay
//check if View in recruiter is present
/* 
  let view_button = browser
  .page
  .query_selector("button[class='artdeco-button artdeco-button--2 artdeco-button--secondary ember-view pvs-profile-actions__action']")
  .await?;

  match view_button {
   Some(view_button) => {
      view_button.hover_builder(); // hover on search input
       wait(1, 4); // random delay
       view_button.click_builder().click().await?; // click on search input
       wait(2, 5); // random delay
   }
   None => {
       wait(1, 5); // random delay
       browser.page.close(Some(false)).await?;
       browser.browser.close().await?; // close browser
       return Err(CustomError::ButtonNotFound("View in recruiter button is not visible".to_string()));
   } // if search input is not found, means page was not loaded and sessuion cookie is not valid
};
*/
let entity_urn = find_entity_run(&browser.page).await?;
let url = format!("https://www.linkedin.com/talent/profile/{}?trk=FLAGSHIP_VIEW_IN_RECRUITER", entity_urn);
// go to candidate page
let mut _go_to = browser
.page
.goto_builder(url.as_str())
.goto()
.await;
let mut x = 0;
if go_to.is_err() {

    while x <= 3 {
        wait(3, 6);
        let build = browser
        .page
        .goto_builder(url.as_str())
        .goto()
        .await;
        if build.is_ok() {
            _go_to = build;
            break;
        } else if build.is_err() && x == 3 {
            wait(3, 6);
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?; // close browser
            return Err(CustomError::ButtonNotFound("Candidate Recruiter page is not loading/Inmail".to_string())); // if error means page is not loading
        }
        x += 1;
        println!("retrying to load page")
    }
    
}

let nav_bar = browser
.page
.query_selector("div[class='global-nav__right']")
.await?;

match &nav_bar {
   Some(_) => (),
   None => {
       wait(1, 3);
       browser.page.close(Some(false)).await?;
       browser.browser.close().await?;
       return Err(CustomError::RecruiterSessionCookieExpired); // if error when session cookie expired
   }
}

wait(50, 600);
wait(2, 4);
let profile_block = browser
.page
.query_selector("div[class='topcard-condensed__content-top topcard-condensed__content-top--profile-size-7']")
.await?;

match &profile_block {
   Some(_) => (),
   None => {
       wait(1, 3);
       browser.page.close(Some(false)).await?;
       browser.browser.close().await?;
       return Err(CustomError::ProfileNotFound); 
   }
}

wait(2, 4);

let send_button = browser
.page
.query_selector("button[class='artdeco-button artdeco-button--circle artdeco-button--muted artdeco-button--2 artdeco-button--tertiary ember-view profile-item-actions__item']")
.await?;

match send_button {
 Some(button) => {
   button.hover_builder(); // hover on search input
     wait(1, 4); // random delay
     button.click_builder().click().await?; // click on search input
     wait(2, 5); // random delay
 }
 None => {
     wait(1, 5); // random delay
     browser.page.close(Some(false)).await?;
     browser.browser.close().await?; // close browser
     return Err(CustomError::ButtonNotFound("Send button in recruiter is not visible/Page".to_string()));
 } 
};

let subject_input = browser
.page
.query_selector("input[class='compose-subject__input']")
.await?;

match subject_input {
 Some(input) => {
   input.hover_builder(); // hover on search input
     wait(1, 4); // random delay
     input.click_builder().click().await?; // click on search input
     wait(2, 5); // random delay
     input.fill_builder(subject.as_str()).fill().await?; // fill input for note;
 }
 None => {
     wait(1, 5); // random delay
     browser.page.close(Some(false)).await?;
     browser.browser.close().await?; // close browser
     return Err(CustomError::ButtonNotFound("Subject input in recruiter is not visible".to_string()));
 } 
};
wait(2, 5);

let text_input = browser
.page
.query_selector("textarea[class='compose-textarea__textarea']")
.await?;

match text_input {
   Some(input) => {
     input.hover_builder(); // hover on search input
       wait(1, 4); // random delay
       input.click_builder().click().await?; // click on search input
       wait(2, 5); // random delay
       input.fill_builder(candidate.message.as_str()).fill().await?; // fill input for note;
   }
   None => {
       wait(1, 5); // random delay
       browser.page.close(Some(false)).await?;
       browser.browser.close().await?; // close browser
       return Err(CustomError::ButtonNotFound("Subject input in recruiter is not visible".to_string()));
   } 
  };

  let send_button = browser
.page
.query_selector("button[class='msg-cmpt__button--small compose-actions__submit-button']")
.await?;

match send_button {
 Some(button) => {
   button.hover_builder(); // hover on search input
     wait(1, 4); // random delay
     button.click_builder().click().await?; // click on search input
     wait(2, 5); // random delay
 }
 None => {
     wait(1, 5); // random delay
     browser.page.close(Some(false)).await?;
     browser.browser.close().await?; // close browser
     return Err(CustomError::ButtonNotFound("Send button in recruiter is not visible/Text".to_string()));
 } 
};


wait(2, 4);
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



