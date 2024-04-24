use super::misc::split_around_dot;
use crate::actions::scrap_profile_f::misc::get_date;
use crate::actions::scrap_profile_f::misc::serialize_option_i64;
use crate::actions::scrap_profile_f::misc::serialize_option_string;
use scraper::ElementRef;
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
    pub skills: Vec<Skill>,
}
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Skill {
    pub order: i8,
    pub skill: Option<String>,
}

use serde::{Deserialize, Serialize, Serializer};
pub fn parse_experience(html_content: &str) -> Vec<Experience> {
    //test_selector(html_content);
    //return vec![];
    let document = Html::parse_document(html_content);
    let experience_selector = Selector::parse(".pvs-list__paged-list-item").unwrap();
    let mut experiences = Vec::new();

    let outer = Selector::parse("#profile-content > div > div.scaffold-layout > div.scaffold-layout__inner > div > main > section > div.pvs-list__container > div > div  > ul");
    //for element in document.select(&outer.unwrap()) {
    //    let html = element.inner_html();
    //    println!("html {}", html);
    //}

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
        let mut experience = Experience::default();
        let hoverable_selector = Selector::parse("div.hoverable-link-text").unwrap();
        if element.select(&hoverable_selector).next().is_some() {
            let exp_vec = parse_each_experience_type_two(element);
            //experiences = experiences.append(&mut exp_vec);
        } else {
            let exp = parse_each_experience_type_one(element, experience);
            //experience = experiences;
        }
    }
    return vec![];
    let experience_selector = Selector::parse("li.pvs-list__paged-list-item").unwrap();

    for _list in document.select(&outer.unwrap()) {
        for element in document.select(&experience_selector) {
            let mut experience = Experience::default();
            let hoverable_selector = Selector::parse("div.hoverable-link-text").unwrap();

            // Determine which snippet type based on the presence of these classes
            if element.select(&hoverable_selector).next().is_some() {
                //let exp = parse_each_experience_type_one(element, experience);
                //experience = exp;
                // Logic for the first snippet
            } else {
                parse_each_experience_type_one(element, experience);
                //experience = exp;
            }

            //company name
            //companyLetter
            //position
            //<employment_type
            //location
            //description

            // Extract company URL
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

            // Extract Skills
            let skills_selector =
                Selector::parse("div.display-flex.t-14.t-normal.t-black > span:nth-of-type(1)")
                    .unwrap();
            let mut skills = Vec::new();
            let mut order = 1; // Initialize a counter for the order of skills

            for skill_element in element.select(&skills_selector) {
                let text = skill_element.text().collect::<Vec<_>>().join(" ");
                if text.contains("·") {
                    for skill_text in text.split(" · ").map(|s| s.trim().to_string()) {
                        // Create a new Skill struct instance with an order and the skill
                        if !skill_text.is_empty() {
                            let skill = Skill {
                                order: order,
                                skill: Some(skill_text),
                            };
                            skills.push(skill);
                            order = order.checked_add(1).expect("Order value overflowed");
                        }
                    }
                }
            }
            experience.skills = skills;

            // Extract period text
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

            // Extract duration
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

            let date = experience.periodText.clone();
            let new_vec = get_date(date.as_deref());
            if new_vec.is_ok() {
                let vect = new_vec.unwrap();
                experience.startDate = vect[0];
                experience.endDate = vect[1];
            }
            experiences.push(experience);
        }
    }

    experiences
}

fn parse_each_experience_type_one(element: ElementRef, mut experience: Experience) -> Experience {
    //println!("HTML ONE: {}", element.inner_html());
    //Extract company name and company letters

    if let Some(company_name) = element
        .select(&Selector::parse("span.t-14.t-normal > span:nth-of-type(1)").unwrap())
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

    //Extract Position
    if let Some(position_element) = element
        .select(&Selector::parse(".t-bold span").unwrap())
        .next()
    {
        experience.position = position_element.text().next().map(String::from);
    }

    //Extract Employment Type
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
        //println!("full text:!!!!!!!!!!!!!{:?}", employment_type);
        let second_half = employment_type.1;
        let result = if second_half.is_some() {
            split_around_dot(second_half.unwrap().as_str())
        } else {
            (None, None)
        };

        experience.employmentType = if result.1.is_some() { result.1 } else { None };
    }

    //Extract location
    experience.location = element
        .select(
            &Selector::parse("span.t-14.t-normal.t-black--light > span:nth-of-type(1)").unwrap(),
        )
        .skip(1)
        .next()
        .map(|e| e.text().collect::<Vec<_>>().join(""));

    // Extract detailed description (if available)
    if let Some(description_div) = element
        .select(&Selector::parse(".display-flex.full-width").unwrap())
        .next()
    {
        experience.description = description_div
            .select(
                &Selector::parse(
                    ".display-flex.align-items-center.t-14.t-normal.t-black > span:nth-of-type(1)",
                )
                .unwrap(),
            )
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_owned());
    }
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
                        order: order,
                        skill: Some(skill_text),
                    };
                    skills.push(skill);
                    order = order.checked_add(1).expect("Order value overflowed");
                }
            }
        }
    }
    experience.skills = skills;

    // Extract period text
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

    // Extract duration
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

    let date = experience.periodText.clone();
    let new_vec = get_date(date.as_deref());
    if new_vec.is_ok() {
        let vect = new_vec.unwrap();
        experience.startDate = vect[0];
        experience.endDate = vect[1];
    }
    println!("One: {:?}", experience);
    experience
}

fn parse_each_experience_type_two(
    element: ElementRef,
    //mut experiences: Vec<Experience>,
) -> Vec<Experience> {
    println!("HTML ONE: {}", element.inner_html());

    //Extract company name and company letters

    let mut company: Option<String> = None;
    let mut employment: Option<String> = None;
    let mut company_id: Option<String> = None;
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
    let company_url_origin = company_url.clone();
    let company_id_origin = if company_url.is_some() {
        Some(
            company_url_origin
                .unwrap()
                .replace(COMPANY_REPLACE, "")
                .replace("/", ""),
        )
    } else {
        None
    };
    company_id = company_id_origin;

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
        //println!("full text:!!!!!!!!!!!!!{:?}", employment_type);
    }
    let position_selector =
        Selector::parse(".optional-action-target-wrapper .display-flex.align-items-center .t-bold")
            .unwrap();
    //Extract location
    for element in element.select(&position_selector).skip(1) {
        let mut experience = Experience::default();
        experience.companyName = company.clone();

        experience.companyLetter = company.clone();
        experience.employmentType = employment.clone();
        experience.companyURL = company_url.clone();
        experience.companyId = company_id.clone();

        experience.logo = company_logo.clone();
        experience.employmentType = employment.clone();
        //Extract Position
        if let Some(position_element) = element
            .select(&Selector::parse(".t-bold span").unwrap())
            .next()
        {
            experience.position = position_element.text().next().map(String::from);
        }

        experience.location = element
            .select(&Selector::parse("span[aria-hidden='true']").unwrap())
            .skip(1)
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join(""));

        // Extract detailed description (if available)
        if let Some(description_div) = element
            .select(
                &Selector::parse("div.display-flex.flex-column.full-width.align-self-center")
                    .unwrap(),
            )
            .next()
        {
            experience.description = description_div
                .select(
                    &Selector::parse("div.display-flex.flex-row.justify-space-between").unwrap(),
                )
                .next()
                .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_owned());
        }
        let selector = Selector::parse("div.display-flex.flex-column.full-width.align-self-center > div.display-flex.flex-row.justify-space-between").unwrap();

        // Select and iterate over nodes matching the selector
        for element in element.select(&selector) {
            // Assuming you want to print the text inside each matching <div>
            let contents = element.text().collect::<Vec<_>>().join(" ");
            println!("Found: {}", contents);
        }
        // Extract Skills
        let skills_selector =
            Selector::parse("div.display-flex.align-items-center.t-14.t-normal.t-black > span")
                .unwrap();
        let mut skills = Vec::new();
        let mut order = 1; // Initialize a counter for the order of skills

        for skill_element in element.select(&skills_selector) {
            let text = skill_element.text().collect::<Vec<_>>().join(" ");
            if text.contains("·") {
                for skill_text in text.split(" · ").map(|s| s.trim().to_string()) {
                    // Create a new Skill struct instance with an order and the skill
                    if !skill_text.is_empty() {
                        let skill = Skill {
                            order: order,
                            skill: Some(skill_text),
                        };
                        skills.push(skill);
                        order = order.checked_add(1).expect("Order value overflowed");
                    }
                }
            }
        }
        experience.skills = skills;

        // Extract period text
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

        // Extract duration
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

        let date = experience.periodText.clone();
        let new_vec = get_date(date.as_deref());
        if new_vec.is_ok() {
            let vect = new_vec.unwrap();
            experience.startDate = vect[0];
            experience.endDate = vect[1];
        }

        //experiences.push(experience);
        //    experiences.push(experience);

        println!("Two {:?}", experience);
    }
    vec![]
}

fn maincheck(html: &str) {
    // Your HTML input goes here. For demonstration, only a small snippet is considered.

    // Parse the HTML document
    let document = Html::parse_document(html);

    // Define selectors for common and individual fields
    let company_selector = Selector::parse(".display-flex .hoverable-link-text.t-bold").unwrap();
    let employment_type_selector = Selector::parse(".t-normal").unwrap();
    let position_selector =
        Selector::parse(".optional-action-target-wrapper .display-flex.align-items-center .t-bold")
            .unwrap();
    let description_selector =
        Selector::parse(".pvs-entity__sub-components .t-normal.t-black").unwrap();
    let duration_selector = Selector::parse(".pvs-entity__caption-wrapper").unwrap();

    // Extract common company name and employment type (assuming they are the same across entries)
    if let Some(company) = document.select(&company_selector).next() {
        println!(
            "Company Name: {}",
            company.text().collect::<Vec<_>>().join("")
        );
    }
    if let Some(employment_type) = document.select(&employment_type_selector).next() {
        println!(
            "Employment Type: {}",
            employment_type.text().collect::<Vec<_>>().join("")
        );
    }

    // Extract individual fields for each position
    for element in document.select(&position_selector) {
        println!(
            "Position Name: {}",
            element.text().collect::<Vec<_>>().join("")
        );

        if let Some(description) = element.select(&description_selector).next() {
            println!(
                "Description: {}",
                description.text().collect::<Vec<_>>().join("")
            );
        }

        if let Some(duration) = element.select(&duration_selector).next() {
            println!("Duration: {}", duration.text().collect::<Vec<_>>().join(""));
        }
    }
}
