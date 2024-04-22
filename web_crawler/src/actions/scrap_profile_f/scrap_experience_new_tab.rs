use super::misc::split_around_dot;
use crate::actions::scrap_profile_f::misc::get_date;
use crate::actions::scrap_profile_f::misc::serialize_option_i64;
use crate::actions::scrap_profile_f::misc::serialize_option_string;
//use crate::actions::scrap_profile_f::misc::split_around_dot;
use scraper::{Html, Selector};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Experience {
    #[serde(serialize_with = "serialize_option_string")]
    pub companyName: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub companyId: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub companyLetter: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub companyURL: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub logo: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub position: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub employmentType: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub periodText: Option<String>,
    #[serde(serialize_with = "serialize_option_i64")]
    pub startDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_i64")]
    pub endDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_string")]
    pub duration: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub location: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    pub description: Option<String>,
}

use serde::{Deserialize, Serialize, Serializer};
pub fn parse_experience(html_content: &str) -> Vec<Experience> {
    let document = Html::parse_document(html_content);
    let experience_selector = Selector::parse(".pvs-list__paged-list-item").unwrap();
    let mut experiences = Vec::new();

    for element in document.select(&experience_selector) {
        let mut experience = Experience::default();

        // Extract company name and URL
        if let Some(a_tag) = element
            .select(&Selector::parse("a.optional-action-target-wrapper").unwrap())
            .next()
        {
            experience.companyURL = a_tag.value().attr("href").map(String::from);
        }
        const COMPANY_REPLACE: &str = "https://www.linkedin.com/company/";
        let company_url = experience.companyURL.clone();
        let company_id = if company_url.is_some() {
            Some(
                company_url
                    .unwrap()
                    .replace(COMPANY_REPLACE, "")
                    .replace("/", ""),
            )
        } else {
            None
        };
        experience.companyId = company_id;

        // Extract logo
        if let Some(img_tag) = element.select(&Selector::parse("img").unwrap()).next() {
            experience.logo = img_tag.value().attr("src").map(String::from);
        }

        // Extract position
        if let Some(position_element) = element
            .select(&Selector::parse(".t-bold span").unwrap())
            .next()
        {
            experience.position = position_element.text().next().map(String::from);
        }

        // Extract employment type, period and location

        if let Some(company_name) = element
            .select(&Selector::parse("span.t-14.t-normal").unwrap())
            .next()
        {
            let full_text = company_name
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let company_name = split_around_dot(&full_text);
            experience.companyName = company_name.0.clone();
            experience.companyLetter = company_name.0;
        }
        if let Some(employment_info) = element
            .select(&Selector::parse("span.t-14.t-normal").unwrap())
            .next()
        {
            let full_text = employment_info
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let employment_type = split_around_dot(&full_text);
            println!("full text:!!!!!!!!!!!!!{:?}", employment_type);
            let second_half = employment_type.1;
            let result = if second_half.is_some() {
                split_around_dot(second_half.unwrap().as_str())
            } else {
                (None, None)
            };

            experience.employmentType = if result.1.is_some() { result.1 } else { None };
        }

        if let Some(period_element) = element
            .select(&Selector::parse("span.pvs-entity__caption-wrapper").unwrap())
            .next()
        {
            let full_text = period_element
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let period_duration = split_around_dot(&full_text);
            experience.periodText = period_duration.0;
        }

        if let Some(duration_element) = element
            .select(&Selector::parse("span.pvs-entity__caption-wrapper").unwrap())
            .next()
        {
            let full_text = duration_element
                .text()
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            let period_duration = split_around_dot(&full_text);
            experience.duration = period_duration.1;
        }

        experience.location = element
            .select(&Selector::parse("span.t-14.t-normal.t-black--light").unwrap())
            .skip(1)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(""));

        // Extract detailed description (if available)
        if let Some(description_div) = element
            .select(&Selector::parse(".pvs-list__outer-container").unwrap())
            .next()
        {
            experience.description = description_div
                .select(&Selector::parse(".pvs-list__item--with-top-padding").unwrap())
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_owned());
        }

        let date = experience.periodText.clone();
        let new_vec = get_date(date.as_deref());
        if new_vec.is_ok() {
            let vect = new_vec.unwrap();
            experience.startDate = vect[0];
            experience.endDate = vect[1];
        }
        experiences.push(experience);
    }

    experiences
}
