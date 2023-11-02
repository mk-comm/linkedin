use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
//use hyper::client::connect::HttpInfo;
use playwright::api::Page;
use scraper::{Html, Selector};
//use serde_json::json;
#[allow(dead_code)]
pub async fn scrap_profile(entry: EntrySendConnection) -> Result<(), CustomError> {
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
        recruiter_session_cookie: None,
        headless: false,
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

    let _entity_urn = match find_entity_run(&browser.page).await {
        Ok(entity_urn) => entity_urn,
        Err(_) => {
            wait(1, 5); // random delay
            browser.page.close(Some(false)).await?;
            browser.browser.close().await?;
            return Err(playwright::Error::InitializationError.into());
        }
    };
    let _data = get_all_linkedin_data(&browser.page.content().await?);

    //println!("entity_urn: {}", entity_urn);
    /*
        let contact_info = browser
            .page
            .query_selector("a#top-card-text-details-contact-info")
            .await?
            .unwrap();
        //let url = contact_info.get_attribute("href").await?;
        //println!("url: {}", url.unwrap());

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
    */
    wait(5, 7);
    browser.page.close(Some(false)).await?;
    browser.browser.close().await?;

    Ok(())
}

async fn find_entity_run(page: &Page) -> Result<String, playwright::Error> {
    let link_selector = Selector::parse("a").unwrap();
    let document = scraper::Html::parse_document(&page.content().await?);
    let mut entity_urn = String::new();

    for link in document.select(&link_selector) {
        let href = link.value().attr("href").unwrap_or_default();
        if href.contains("profileUrn=") {
            let parts: Vec<&str> = href
                .split("?profileUrn=urn%3Ali%3Afsd_profile%3A")
                .collect();
            if parts.len() > 1 {
                entity_urn = parts[1].split("&").collect::<Vec<&str>>()[0].to_string();
                if entity_urn.is_empty() {
                    let parts: Vec<&str> = href
                        .split("?profileUrn=urn%3Ali%3Afs_normalized_profile%3A")
                        .collect();
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

    if entity_urn.is_empty() {
        entity_urn = print_elements_with_datalet_in_id(document.html().as_str());
    }
    Ok(entity_urn)
}

fn print_elements_with_datalet_in_id(html: &str) -> String {
    // Parse the document
    let document = Html::parse_document(html);

    // Create a Selector for elements with an 'id' attribute
    let selector = Selector::parse("[id]").unwrap();

    let mut right_id = String::new();
    // Iterate over elements matching the selector
    for element in document.select(&selector) {
        if let Some(id_attr) = element.value().attr("id") {
            if id_attr.contains("datalet")
                && element
                    .html()
                    .contains("/voyager/api/identity/dash/profile")
            {
                let element_html: String = element.html();
                match element_html.find("bpr-guid-") {
                    Some(start) => match element_html[start..].find("\"") {
                        Some(end) => {
                            let end = end + start;
                            right_id = format!("[id={}]", &element_html[start..end]);
                        }
                        None => println!("Could not find end quote"),
                    },
                    None => println!("Could not find 'bpr-guid-'"),
                }
            }
        }
    }

    let entity_id_selector = Selector::parse(&right_id).unwrap();
    let mut entity_urn = String::new();
    for element in document.select(&entity_id_selector) {
        let text = element.html();
        let text_str = text.as_str();

        if let Some(start) = text_str.find("\"*elements\":[\"urn:li:fsd_profile:") {
            let start = start + "\"*elements\":[\"urn:li:fsd_profile:".len();
            if let Some(end) = text_str[start..].find("\"") {
                let end = start + end;
                entity_urn = text_str[start..end].to_string();
            }
        }
    }

    entity_urn
}

fn get_all_linkedin_data(html: &str) -> Result<(), CustomError> {
    /*
        let location = "https://www.linkedin.com"; // replace with actual location
        let mut linkedin = String::new();
        let mut linkedin_unique = String::new();
        let mut linkedin_unique_number = 0;
        let mut entity_urn = String::new();
        let mut connection_level = 1000;
        let mut city_location = String::new();
        let mut experience = String::new();
        let mut education = String::new();
        let mut certifications = String::new();
        let mut languages = String::new();
        let mut about = String::new();
        let mut profile_picture = String::new();
        let mut viewed_in = String::new();
        let language = "en"; // replace with actual language
        let properties_to_translate = vec!["education.periodtext", "certifications.periodtex", "experience.periodtext", "experience.employmenttype", "experience.duration"];
    */

    let job = "Software Engineer"; // replace with actual job
    let first = "John"; // replace with actual first name
    let last = "Doe"; // replace with actual last name
    let email = "john.doe@example.com"; // replace with actual email
    let title = "Software Engineer"; // replace with actual title
    let company = "Company"; // replace with actual company
    let company_unique = "Company Unique"; // replace with actual company unique

    let _document = Html::parse_document(&html);

    // Create the selectors
    /*
    let job_selector = Selector::parse("#tribexyz_widget_job").unwrap();
    let first_selector = Selector::parse("#tribexyz_widget_first_name").unwrap();
    let last_selector = Selector::parse("#tribexyz_widget_last_name").unwrap();
    let email_selector = Selector::parse("#tribexyz_widget_email").unwrap();
    let title_selector = Selector::parse("#tribexyz_widget_title").unwrap();
    let company_selector = Selector::parse("#tribexyz_widget_company").unwrap();
    let company_unique_selector = Selector::parse("#tribexyz_widget_company_unique").unwrap();

    // Extract the data
    let job = document.select(&job_selector).next().unwrap().value().attr("value").unwrap();
    let first = document.select(&first_selector).next().unwrap().value().attr("value").unwrap();
    let last = document.select(&last_selector).next().unwrap().value().attr("value").unwrap();
    let email = document.select(&email_selector).next().unwrap().value().attr("value").unwrap();
    let title = document.select(&title_selector).next().unwrap().value().attr("value").unwrap();
    let company = document.select(&company_selector).next().unwrap().value().attr("value").unwrap();
    let company_unique = document.select(&company_unique_selector).next().unwrap().value().attr("value").unwrap();
    */
    let experience = get_experience_for_full_normal_mode(html);
    print!("experience: {:?}", experience.unwrap());
    //let certification = get_certifications_for_full_normal_mode(html);
    //let education = get_education_for_normal_mode(html);
    //let about = get_about_for_normal_mode(html);
    let profile_picture = get_profile_picture_for_normal_mode(html);
    print!("job: {}", job);
    print!("first: {}", first);
    print!("last: {}", last);
    print!("email: {}", email);
    print!("title: {}", title);
    print!("company: {}", company);
    print!("company_unique: {}", company_unique);

    //print!("certification: {:?}", certification);
    //print!("education: {:?}", education);
    //print!("about: {:?}", about);
    print!("profile_picture: {:?}", profile_picture.unwrap());

    Ok(())
}
#[allow(dead_code)]
fn get_experience_for_full_normal_mode(html: &str) -> Result<Vec<Experience>, CustomError> {
    let _location = "https://www.linkedin.com"; // replace with actual location

    // Load the HTML document
    let document = Html::parse_document(&html);

    // Create the selectors
    let experience_section_selector =
        Selector::parse("#experience.pv-profile-card__anchor").unwrap();
    let experience_boxes_selector = Selector::parse(
        "li[class='artdeco-list__item pvs-list__item--line-separated pvs-list__item--one-column']",
    )
    .unwrap();

    // Extract the data
    let experience_section = document
        .select(&experience_section_selector)
        .next()
        .unwrap();
    let experience_boxes = experience_section.select(&experience_boxes_selector);

    let mut experience = Vec::new();

    let _period_text_selector = Selector::parse(".t-14.t-normal.t-black--light").unwrap();
    let _employment_type_selector = Selector::parse(".t-14.t-normal").unwrap();
    let _company_name_selector =
        Selector::parse(".optional-action-target-wrapper .EntityPhoto-square-3").unwrap();
    let _company_id_selector = Selector::parse("a[data-field='experience_company_logo']").unwrap();
    let _job_title_selector =
        Selector::parse(".display-flex.align-items-center.mr1.t-bold").unwrap();
    let _description_selector = Selector::parse(".pv-shared-text-with-see-more").unwrap();
    let _start_date_selector = Selector::parse(".t-14.t-normal.t-black--light").unwrap();
    let _end_date_selector = Selector::parse(".t-14.t-normal.t-black--light").unwrap();
    let _duration_selector = Selector::parse(".t-14.t-normal.t-black--light").unwrap();

    for experience_box in experience_boxes {
        let period_text = experience_box
            .value()
            .attr("data-period-text")
            .unwrap_or("");
        let employment_type = experience_box
            .value()
            .attr("data-employment-type")
            .unwrap_or("");
        let company_name = experience_box
            .value()
            .attr("data-company-name")
            .unwrap_or("");
        let company_id = experience_box.value().attr("data-company-id").unwrap_or("");
        let job_title = experience_box.value().attr("data-job-title").unwrap_or("");
        let description = experience_box
            .value()
            .attr("data-description")
            .unwrap_or("");
        let start_date = experience_box.value().attr("data-start-date").unwrap_or("");
        let end_date = experience_box.value().attr("data-end-date").unwrap_or("");
        let duration = experience_box.value().attr("data-duration").unwrap_or("");

        let item = Experience {
            period_text: period_text.to_string(),
            employment_type: employment_type.to_string(),
            company_name: company_name.to_string(),
            company_id: company_id.to_string(),
            job_title: job_title.to_string(),
            description: description.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            duration: duration.to_string(),
        };
        print!("{:?}", experience);
        experience.push(item);
    }

    Ok(experience)
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Experience {
    period_text: String,
    employment_type: String,
    company_name: String,
    company_id: String,
    job_title: String,
    description: String,
    start_date: String,
    end_date: String,
    duration: String,
}
#[allow(dead_code)]
fn get_certifications_for_full_normal_mode(html: &str) -> Result<Vec<Certification>, CustomError> {
    let _location = "https://www.linkedin.com"; // replace with actual location

    // Load the HTML document
    let document = Html::parse_document(&html);

    // Create the selectors
    let certifications_section_selector = Selector::parse("div[id=certifications]").unwrap();
    let certificate_boxes_selector = Selector::parse("div.certificate-section__item").unwrap();

    // Extract the data
    let certifications_section = document
        .select(&certifications_section_selector)
        .next()
        .unwrap();
    let certificate_boxes = certifications_section.select(&certificate_boxes_selector);

    let mut certifications = Vec::new();

    for certificate_box in certificate_boxes {
        let period_text = certificate_box
            .value()
            .attr("data-period-text")
            .unwrap_or("");
        let name = certificate_box.value().attr("data-name").unwrap_or("");
        let description = certificate_box
            .value()
            .attr("data-description")
            .unwrap_or("");

        let item = Certification {
            period_text: period_text.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        };

        certifications.push(item);
    }

    Ok(certifications)
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Certification {
    period_text: String,
    name: String,
    description: String,
}
#[allow(dead_code)]
fn get_education_for_normal_mode(html: &str) -> Result<Vec<Education>, CustomError> {
    let _location = "https://www.linkedin.com"; // replace with actual location

    // Load the HTML document
    let document = Html::parse_document(&html);

    // Create the selectors
    let education_section_selector = Selector::parse("section.pv-education-section").unwrap();
    let education_boxes_selector = Selector::parse("div.education-section__item").unwrap();

    // Extract the data
    let education_section = document.select(&education_section_selector).next().unwrap();
    let education_boxes = education_section.select(&education_boxes_selector);

    let mut education = Vec::new();

    for education_box in education_boxes {
        let school_name = education_box.value().attr("data-school-name").unwrap_or("");
        let degree = education_box.value().attr("data-degree").unwrap_or("");
        let field_of_study = education_box
            .value()
            .attr("data-field-of-study")
            .unwrap_or("");
        let start_date = education_box.value().attr("data-start-date").unwrap_or("");
        let end_date = education_box.value().attr("data-end-date").unwrap_or("");
        let description = education_box.value().attr("data-description").unwrap_or("");

        let item = Education {
            school_name: school_name.to_string(),
            degree: degree.to_string(),
            field_of_study: field_of_study.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            description: description.to_string(),
        };

        education.push(item);
    }

    Ok(education)
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Education {
    school_name: String,
    degree: String,
    field_of_study: String,
    start_date: String,
    end_date: String,
    description: String,
}
#[allow(dead_code)]
fn get_about_for_normal_mode(html: &str) -> Result<String, CustomError> {
    let _location = "https://www.linkedin.com"; // replace with actual location

    // Load the HTML document
    let document = Html::parse_document(&html);

    // Create the selector
    let about_box_selector = Selector::parse("section.pv-about-section div").unwrap();

    // Extract the data
    let about_box = document.select(&about_box_selector).next().unwrap();
    let about_text = about_box.text().collect::<Vec<_>>().join("");

    Ok(about_text)
}

fn get_profile_picture_for_normal_mode(html: &str) -> Result<String, CustomError> {
    let _location = "https://www.linkedin.com"; // replace with actual location

    // Load the HTML document
    let document = Html::parse_document(&html);

    // Create the selector
    let img_src_selector = Selector::parse(".pv-top-card-profile-picture__image").unwrap();

    // Extract the data
    let img_src = document
        .select(&img_src_selector)
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap_or("");

    if img_src.starts_with("data:image/gif;base64") {
        return Ok(String::new());
    }

    Ok(img_src.to_string())
}
