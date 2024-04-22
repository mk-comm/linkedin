use super::misc::split_around_dot;
use super::scrap_experience_new_tab::Experience;
use crate::actions::scrap_profile_f::misc::get_date;
use crate::actions::scrap_profile_f::misc::serialize_option_i64;
use crate::actions::scrap_profile_f::misc::serialize_option_string;
//use crate::actions::scrap_profile_f::misc::split_around_dot;
use scraper::{Html, Selector};

use crate::structs::error::CustomError;
use playwright::api::Page;
use serde::{Deserialize, Serialize, Serializer};

pub async fn parse_experience_same_page(page: Page) -> Result<Vec<Experience>, CustomError> {
    const MAIN_SELECTOR: &str = "main[class='scaffold-layout__main']";

    const INNER_TABLE_SELECTOR: &str = "ul[class='pvs-list
            
            
            ']";
    const EXPERIENCE_TABLE_SELECTOR: &str = "div[class='pvs-list__outer-container']";
    let main = page.query_selector(MAIN_SELECTOR).await?;
    let experience_list = match main {
        Some(element) => element.query_selector(EXPERIENCE_TABLE_SELECTOR).await?,
        None => {
            println!("main None");
            None
        }
    };
    let html = match experience_list {
        Some(element) => element.inner_html().await?,
        None => {
            return Err(CustomError::ButtonNotFound(
                "Candidate page is not loading/Connection".to_string(),
            ))
        }
    };
    println!("html: {:?}", html);
    let exp_vec = parse_each_experience(html.as_str());

    let exp_vec_1 = parse_each_experience(html.as_str());
    for exp in exp_vec {
        println!("{:?}", exp)
    }

    Ok(exp_vec_1)
}

pub fn parse_each_experience(html_content: &str) -> Vec<Experience> {
    println!("started pares {}", html_content);
    let document = Html::parse_document(html_content);
    let experience_selector = Selector::parse(".pvs-list__paged-list-item").unwrap();
    let mut experiences = Vec::new();

    for element in document.select(&experience_selector) {
        let mut experience = Experience::default();

        // Extract company URL and company ID from 'a' tag if available
        if let Some(a_tag) = element
            .select(&Selector::parse("a.optional-action-target-wrapper").unwrap())
            .next()
        {
            if let Some(href) = a_tag.value().attr("href") {
                experience.companyURL = Some(href.to_string());
                // Assuming the company ID is the last segment of the URL after the last '/'
                experience.companyId = href.split('/').last().map(String::from);
            }
        }

        // Extract logo from 'img' tag if available
        if let Some(img_tag) = element.select(&Selector::parse("img").unwrap()).next() {
            if let Some(src) = img_tag.value().attr("src") {
                experience.logo = Some(src.to_string());
            }
        }

        // Extract position
        if let Some(position_span) = element
            .select(&Selector::parse(".t-bold span").unwrap())
            .next()
        {
            experience.position = position_span.text().next().map(String::from);
        }

        // Extract company name and employment type
        if let Some(company_info_span) = element
            .select(&Selector::parse("span.t-14.t-normal").unwrap())
            .next()
        {
            let full_text = company_info_span.text().collect::<Vec<_>>().join("");
            let (company_name, employment_type) = split_around_dot(&full_text);
            experience.companyName = company_name;
            experience.employmentType = employment_type;
        }

        // Extract period text and location
        if let Some(period_span) = element
            .select(&Selector::parse("span.t-14.t-normal.t-black--light").unwrap())
            .next()
        {
            let period_text = period_span.text().collect::<Vec<_>>().join("");
            let (period, location) = split_around_dot(&period_text);
            experience.periodText = period;
            experience.location = location;
        }

        // Description can be more complicated due to the potential for multiple text nodes and formatting
        if let Some(description_div) = element
            .select(&Selector::parse(".pv-shared-text-with-see-more").unwrap())
            .next()
        {
            experience.description = Some(
                description_div
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string(),
            );
        }

        // Parsing dates and additional details can be done here

        experiences.push(experience);
    }

    experiences
}
