use super::misc::split_around_comma;
use crate::actions::scrap_profile::misc::get_date;
use crate::actions::scrap_profile::misc::serialize_option_i64;
use crate::actions::scrap_profile::misc::serialize_option_string;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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
#[allow(non_snake_case)]
pub fn parse_education(html_content: &str) -> Vec<Education> {
    let html = find_education(html_content);
    let html = if let Some(html) = html {
        html
    } else {
        return vec![];
    };
    let document = Html::parse_document(html.as_str());
    let education_selector = Selector::parse("li.artdeco-list__item").unwrap();
    let mut educations = Vec::new();

    for element in document.select(&education_selector) {
        let id = extract_id(element.html().as_str());
        //let id = element.value().id().map(String::from);
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

        //Extract Start Date & End Date
        let date = periodText.clone();
        let new_vec = get_date(date.as_deref());
        let second_vec = get_date(date.as_deref());
        let start_date = if second_vec.is_ok() {
            let vect = second_vec.unwrap();
            vect[0]
        } else {
            None
        };
        let end_date = if new_vec.is_ok() {
            let vect = new_vec.unwrap();
            vect[1]
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
            startDate: start_date,
            endDate: end_date,
            description,
        });
    }

    educations
}

fn extract_id(html_content: &str) -> Option<String> {
    let document = Html::parse_fragment(html_content);
    // Selector for a section with the class 'artdeco-card' and containing the 'data-member-id' attribute
    let selector = Selector::parse("a.optional-action-target-wrapper.display-flex[href]").unwrap();

    // Attempt to find the section element and extract the 'data-member-id' attribute
    let url = document
        .select(&selector)
        .next()
        .and_then(|section| section.value().attr("href").map(String::from));

    let url_link = if let Some(url_link) = url {
        url_link
    } else {
        return None;
    };

    if url_link.contains("search/results") {
        None
    } else {
        Some(
            url_link
                .replace("https://www.linkedin.com/company/", "")
                .replace("/", ""),
        )
    }
}

fn find_education(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Selector for sections with class `artdeco-card pv-profile-card break-words`
    let section_selector =
        Selector::parse("section.artdeco-card.pv-profile-card.break-words").unwrap();
    // Selector for div with id `education`
    let education_div_selector = Selector::parse("div#education.pv-profile-card__anchor").unwrap();

    for section in document.select(&section_selector) {
        if section.select(&education_div_selector).next().is_some() {
            // Found the section containing the education div, print its HTML
            //println!("found sections {}", section.html());
            return Some(section.html());
        }
    }

    None
}
