use super::misc::split_around_comma;
use crate::actions::start_browser::start_browser;
use crate::actions::wait::wait;
use crate::structs::browser::BrowserInit;
use crate::structs::candidate::Candidate;
use crate::structs::entry::EntrySendConnection;
use crate::structs::error::CustomError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};
//use hyper::client::connect::HttpInfo;
use crate::actions::scrap_profile_f::misc::serialize_option_i64;
use crate::actions::scrap_profile_f::misc::serialize_option_string;
use playwright::api::Page;
use scraper::{Html, Selector};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Education {
    #[serde(serialize_with = "serialize_option_string")]
    id: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    schoolName: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    logoUrl: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    degreeName: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    fieldOfStudy: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    periodText: Option<String>,
    #[serde(serialize_with = "serialize_option_i64")]
    startDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_i64")]
    endDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_string")]
    description: Option<String>,
}

pub fn parse_education(html_content: &str) -> Vec<Education> {
    let document = Html::parse_document(html_content);
    let education_selector = Selector::parse("li.pvs-list__paged-list-item").unwrap();
    let mut educations = Vec::new();

    for element in document.select(&education_selector) {
        let id = element.value().id().map(String::from);
        let schoolName = if let Some(school_element) = element
            .select(
                &Selector::parse(
                    "div[class='display-flex flex-wrap align-items-center full-height']",
                )
                .unwrap(),
            )
            .next()
        {
            Some(
                school_element
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string(),
            )
        } else {
            None
        };

        let logoUrl = if let Some(img) = element.select(&Selector::parse("img").unwrap()).next() {
            img.value().attr("src").map(String::from)
        } else {
            None
        };

        let degreeName = if let Some(span) = element
            .select(&Selector::parse("span.t-14.t-normal > span:nth-of-type(1)").unwrap())
            .next()
        {
            Some(span.text().collect::<Vec<_>>().join(""))
        } else {
            None
        };

        let (degreeName, fieldOfStudy) = match degreeName {
            Some(name) => split_around_comma(&name),
            None => (None, None),
        };

        let url = if let Some(a_tag) = element
            .select(
                &Selector::parse(
                    "a.optional-action-target-wrapper.display-flex.flex-column.full-width",
                )
                .unwrap(),
            )
            .next()
        {
            a_tag.value().attr("href").map(String::from)
        } else {
            None
        };

        let periodText = if let Some(period) = element
            .select(
                &Selector::parse("span.t-14.t-normal.t-black--light > span:nth-of-type(1)")
                    .unwrap(),
            )
            .next()
        {
            Some(period.text().collect::<Vec<_>>().join(""))
        } else {
            None
        };

        let description = Some(String::new());
        educations.push(Education {
            id,
            schoolName,
            logoUrl,
            degreeName,
            fieldOfStudy,
            periodText,
            startDate: None,
            endDate: None,
            description,
        });
    }

    educations
}
