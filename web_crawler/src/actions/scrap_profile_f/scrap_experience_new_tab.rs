use super::misc::split_around_dot;
use crate::actions::scrap_profile_f::misc::get_date;
use crate::actions::scrap_profile_f::misc::serialize_option_i64;
use crate::actions::scrap_profile_f::misc::serialize_option_string;
use scraper::ElementRef;
//use crate::actions::scrap_profile_f::misc::split_around_dot;
use scraper::{Html, Selector};

#[allow(non_snake_case)]
struct ExperiencePartial {
    companyName: Option<String>,
    employmentType: Option<String>,
    companyId: Option<String>,
    companyURL: Option<String>,
    logo: Option<String>,
}

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
    pub skills: Vec<Skill>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Skill {
    pub order: i8,
    pub skill: Option<String>,
}

use serde::{Deserialize, Serialize};
pub fn parse_experience(html_content: &str) -> Vec<Experience> {
    let document = Html::parse_document(html_content);
    let mut experiences = Vec::new();

    let outer = Selector::parse("#profile-content > div > div.scaffold-layout > div.scaffold-layout__inner > div > main > section > div.pvs-list__container > div > div  > ul");

    let mut li_elements: Vec<ElementRef> = Vec::new();

    if let Some(ul) = document.select(&outer.unwrap()).next() {
        li_elements = ul
            .children()
            .filter_map(|element| {
                if element.value().is_element() {
                    ElementRef::wrap(element).filter(|e| e.value().name() == "li")
                } else {
                    None
                }
            })
            .collect();
    }

    for element in li_elements {
        let hoverable_selector = Selector::parse("div.hoverable-link-text").unwrap();
        if element.select(&hoverable_selector).next().is_some() {
            let partial = parse_partial(element);
            let selector =
                Selector::parse("div > div > div > div > ul > li > div > div > div > ul").unwrap();
            let mut li_elements: Vec<ElementRef> = Vec::new();

            if let Some(ul) = document.select(&selector).next() {
                li_elements = ul
                    .children()
                    .filter_map(|element| {
                        if element.value().is_element() {
                            ElementRef::wrap(element).filter(|e| e.value().name() == "li")
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            for el in li_elements {
                let experience = parse_each_experience_type_two(el.inner_html().as_str(), &partial);
                println!("TWO Experience: {:?}", experience);
                experiences.push(experience);
            }
        } else {
            let experience = parse_each_experience_type_one(element);
            println!("One Experience: {:?}", experience);
            experiences.push(experience);
        }
    }

    experiences
}

fn parse_each_experience_type_one(element: ElementRef) -> Experience {
    //Extract company name and company letters
    let company_name = if let Some(company_tuple) = element
        .select(&Selector::parse("div > div > div > div > div > span.t-14 > span").unwrap())
        .next()
    {
        let full_text = company_tuple
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        if split_around_dot(&full_text).0.is_some() {
            split_around_dot(&full_text).0
        } else {
            Some(full_text)
        }
    } else {
        None
    };

    //Extract Position
    let position = if let Some(position_element) = element
        .select(&Selector::parse(".t-bold span").unwrap())
        .next()
    {
        position_element.text().next().map(String::from)
    } else {
        None
    };

    //Extract Employment Type
    let employment_type = if let Some(employment_info) = element
        .select(&Selector::parse("span.t-14.t-normal").unwrap())
        .next()
    {
        let full_text = employment_info
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        let employment_type_extra = split_around_dot(&full_text);
        //println!("full text:!!!!!!!!!!!!!{:?}", employment_type);
        let second_half = employment_type_extra.1;
        let result = if second_half.is_some() {
            split_around_dot(second_half.unwrap().as_str())
        } else {
            (None, None)
        };

        if result.1.is_some() {
            result.1
        } else {
            None
        }
    } else {
        None
    };
    //Extract location
    let location = element
        .select(
            &Selector::parse("span.t-14.t-normal.t-black--light > span:nth-of-type(1)").unwrap(),
        )
        .skip(1)
        .next()
        .map(|e| e.text().collect::<Vec<_>>().join(""));

    // Extract detailed description (if available)
    let description = if let Some(description_div) = element
        .select(&Selector::parse(".display-flex.full-width").unwrap())
        .next()
    {
        description_div
            .select(
                &Selector::parse(
                    ".display-flex.align-items-center.t-14.t-normal.t-black > span:nth-of-type(1)",
                )
                .unwrap(),
            )
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_owned())
    } else {
        None
    };

    // Extract Company Url
    let company_url = if let Some(a_tag) = element
        .select(&Selector::parse("a.optional-action-target-wrapper").unwrap())
        .next()
    {
        a_tag.value().attr("href").map(String::from)
    } else {
        None
    };

    // Extract Company ID
    const COMPANY_REPLACE: &str = "https://www.linkedin.com/company/";
    const SEARCH: &str = "com/search";

    let company_id = if company_url.is_some() {
        let company = company_url.clone().unwrap();
        if !company.contains(SEARCH) {
            Some(
                company
                    .replace(COMPANY_REPLACE, "")
                    .replace("/", "")
                    .to_string(),
            )
        } else {
            None
        }
    } else {
        None
    };

    // Extract logo
    let logo = if let Some(img_tag) = element.select(&Selector::parse("img").unwrap()).next() {
        img_tag.value().attr("src").map(String::from)
    } else {
        None
    };

    // Extract Skills
    let skills_selector =
        Selector::parse("div.display-flex.t-14.t-normal.t-black > span:nth-of-type(1)").unwrap();
    let mut skills = Vec::new();
    let mut order = 1; // Initialize a counter for the order of skills

    for skill_element in element.select(&skills_selector) {
        let text = skill_element.text().collect::<Vec<_>>().join(" ");
        if text.contains("·") {
            for skill_text in text.split(" · ").map(|s| s.trim().to_string()) {
                // Create a new Skill struct instance with an order and the skill
                if !skill_text.is_empty() {
                    let skill = Skill {
                        order,
                        skill: Some(skill_text),
                    };
                    skills.push(skill);
                    order = order.checked_add(1).expect("Order value overflowed");
                }
            }
        }
    }

    // Extract period text
    let period_text = if let Some(period_element) = element
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
        period_duration.0
    } else {
        None
    };

    // Extract duration
    let duration = if let Some(duration_element) = element
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
        period_duration.1
    } else {
        None
    };

    //Extract Start Date & End Date
    let date = period_text.clone();
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
    Experience {
        companyName: company_name.clone(),
        companyId: company_id.clone(),
        companyLetter: company_name.clone(),
        companyURL: company_url.clone(),
        logo,
        position,
        employmentType: employment_type,
        periodText: period_text,
        startDate: start_date,
        endDate: end_date,
        duration,
        location,
        description,
        skills,
    }
}

fn parse_partial(element: ElementRef) -> ExperiencePartial {
    let mut company: Option<String> = None;
    let mut employment: Option<String> = None;
    let mut company_url: Option<String> = None;
    let mut company_logo: Option<String> = None;

    // extract logo
    if let Some(img_tag) = element.select(&Selector::parse("img").unwrap()).next() {
        company_logo = img_tag.value().attr("src").map(String::from);
    }

    if let Some(a_tag) = element
        .select(&Selector::parse("a.optional-action-target-wrapper").unwrap())
        .next()
    {
        company_url = a_tag.value().attr("href").map(String::from);
    }

    const COMPANY_REPLACE: &str = "https://www.linkedin.com/company/";
    const SEARCH: &str = "com/search";

    let company_id = if company_url.is_some() {
        let company = company_url.clone().unwrap();
        if !company.contains(SEARCH) {
            Some(
                company
                    .replace(COMPANY_REPLACE, "")
                    .replace("/", "")
                    .to_string(),
            )
        } else {
            None
        }
    } else {
        None
    };

    if let Some(company_name) = element
        .select(
            &Selector::parse(".display-flex .hoverable-link-text.t-bold > span:nth-of-type(1)")
                .unwrap(),
        )
        .next()
    {
        let full_text = company_name
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        company = Some(full_text);
    }

    //Extract Employment Type
    if let Some(employment_info) = element
        .select(&Selector::parse(".t-normal > span:nth-of-type(1)").unwrap())
        .next()
    {
        let full_text = employment_info
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        let employment_type = split_around_dot(&full_text);
        employment = employment_type.0;
    }

    ExperiencePartial {
        companyName: company,
        companyId: company_id,
        companyURL: company_url,
        logo: company_logo,
        employmentType: employment,
    }
}

fn parse_each_experience_type_two(html: &str, partial: &ExperiencePartial) -> Experience {
    let document = Html::parse_document(html);

    //Extract Position
    let position = if let Some(position_element) = document
        .select(&Selector::parse(".t-bold span").unwrap())
        .next()
    {
        position_element.text().next().map(String::from)
    } else {
        None
    };

    // Selector for description
    let description_selector = Selector::parse(
        "div.display-flex.align-items-center.t-14.t-normal.t-black > span:nth-of-type(1)",
    )
    .unwrap();
    let description = if let Some(element) = document.select(&description_selector).next() {
        Some(element.text().collect::<Vec<_>>().join(""))
    } else {
        None
    };

    // Selector for duration
    let duration_selector =
        Selector::parse("span.t-14.t-normal.t-black--light > span:nth-of-type(1)").unwrap();
    let duration = if let Some(element) = document.select(&duration_selector).next() {
        Some(element.text().collect::<Vec<_>>().join(""))
    } else {
        None
    };
    let period_text = match duration.clone() {
        Some(period) => {
            let period = split_around_dot(&period);
            period.0
        }
        None => None,
    };
    let duration_text = match duration {
        Some(ref duration) => {
            let duration = split_around_dot(&duration);
            duration.1
        }
        None => None,
    };

    //Extract Start Date & End Date
    let date = period_text.clone();
    let new_vec = get_date(date.as_deref());
    let second_vec = get_date(date.as_deref());

    let start_date = if new_vec.is_ok() {
        let vect = new_vec.unwrap();
        vect[0]
    } else {
        None
    };
    let end_date = if second_vec.is_ok() {
        let vect = second_vec.unwrap();
        vect[1]
    } else {
        None
    };

    // Selector for skills
    let skills_selector = Selector::parse("li.pvs-list__item--with-top-padding div.display-flex div.display-flex.align-items-center.t-14.t-normal.t-black").unwrap();
    let skills = document
        .select(&skills_selector)
        .filter_map(|element| {
            let text = element.text().collect::<Vec<_>>().join("");
            if text.contains("Skills:") {
                Some(
                    text.split("Skills:")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .next();
    let mut skills_vec: Vec<Skill> = Vec::new();
    let mut order = 1;
    if skills.is_some() {
        let s = skills.unwrap();
        for skill_text in s.split(" · ").map(|s| s.trim().to_string()) {
            // Create a new Skill struct instance with an order and the skill
            if !skill_text.is_empty() {
                let skill = Skill {
                    order,
                    skill: Some(skill_text),
                };
                order = order.checked_add(1).expect("Order value overflowed");
                skills_vec.push(skill)
            }
        }
    }

    Experience {
        companyName: partial.companyName.clone(),
        companyId: partial.companyId.clone(),
        companyLetter: partial.companyName.clone(),
        companyURL: partial.companyURL.clone(),
        logo: partial.logo.clone(),
        position,
        employmentType: partial.employmentType.clone(),
        periodText: period_text,
        startDate: start_date,
        endDate: end_date,
        duration: duration_text,
        location: None,
        description,
        skills: skills_vec,
    }
}
