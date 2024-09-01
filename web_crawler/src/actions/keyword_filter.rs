use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

use std::fmt;
#[derive(Debug, Deserialize, Serialize)]
pub struct EntryKeyword {
    filters: Vec<Filters>,
    linkedin_sections: Response,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    status: String,
    response: ResponseData,
}
#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    sections: Vec<LinkedinSection>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LinkedinSection {
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub linkedin_sections: LinkedinSections,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LinkedinSections {
    #[serde(rename = "Any LinkedIn profile section")]
    AnyLinkedinProfileSection,
    Headline,
    #[serde(rename = "Current position")]
    CurrentPosition,
    Education,
    Summary,
    Experience,
    Skills,
    Organizations,
}
fn deserialize_linkedin_sections<'de, D>(deserializer: D) -> Result<Vec<LinkedinSections>, D::Error>
where
    D: Deserializer<'de>,
{
    struct LinkedinSectionsVisitor;

    impl<'de> Visitor<'de> for LinkedinSectionsVisitor {
        type Value = Vec<LinkedinSections>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing LinkedIn sections separated by commas")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .split(',')
                .map(|s| s.trim())
                .map(LinkedinSections::from_str)
                .collect::<Result<Vec<_>, _>>()
                .map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(LinkedinSectionsVisitor)
}
impl FromStr for LinkedinSections {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Any LinkedIn profile section" => Ok(LinkedinSections::AnyLinkedinProfileSection),
            "Headline" => Ok(LinkedinSections::Headline),
            "Current position" => Ok(LinkedinSections::CurrentPosition),
            "Education" => Ok(LinkedinSections::Education),
            "Summary" => Ok(LinkedinSections::Summary),
            "Experience" => Ok(LinkedinSections::Experience),
            "Skills" => Ok(LinkedinSections::Skills),
            "Organizations" => Ok(LinkedinSections::Organizations),
            _ => Err(format!("Unknown LinkedIn section: {}", s)),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum InclusionIndicator {
    #[serde(rename = "is contained")]
    IsContained,
    #[serde(rename = "is not contained")]
    NotContained,
}
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Condition {
    AND,
    OR,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Filters {
    #[serde(deserialize_with = "deserialize_keywords")]
    pub keywords: Vec<String>,
    #[serde(deserialize_with = "deserialize_linkedin_sections")]
    pub linkedin_sections: Vec<LinkedinSections>,
    pub inclusion_indicator: InclusionIndicator,
    pub condition: Condition,
}
fn deserialize_keywords<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeywordsVisitor;

    impl<'de> Visitor<'de> for KeywordsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing keywords separated by commas")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.split(',').map(|s| s.trim().to_string()).collect())
        }
    }

    deserializer.deserialize_str(KeywordsVisitor)
}
#[derive(Debug)]
pub struct FilterGroup {
    filters: Vec<Filters>,
}
pub async fn keyword(entry: EntryKeyword) -> bool {
    let filters = entry.filters;
    let sections = entry.linkedin_sections.response.sections;
    let group_filters = group_filters(filters);

    let result = check_keywords_in_groups(group_filters, sections);
    result.is_some()
}

fn group_filters(filters: Vec<Filters>) -> Vec<FilterGroup> {
    let mut groups = vec![];
    let mut current_group = vec![];

    for filter in filters {
        current_group.push(filter);

        if let Condition::OR = current_group.last().unwrap().condition {
            groups.push(FilterGroup {
                filters: current_group,
            });
            current_group = vec![];
        }
    }

    if !current_group.is_empty() {
        groups.push(FilterGroup {
            filters: current_group,
        });
    }

    groups
}

fn check_keywords_in_groups(
    groups: Vec<FilterGroup>,
    linkedin_sections: Vec<LinkedinSection>,
) -> Option<FilterGroup> {
    for group in groups {
        let mut all_filters_matched = true;
        let mut any_filter_matched = false;

        for filter in &group.filters {
            let mut filter_matched = true;

            for section in &filter.linkedin_sections {
                if let Some(linkedin_section) = linkedin_sections
                    .iter()
                    .find(|&ls| &ls.linkedin_sections == section)
                {
                    let keywords_present = filter.keywords.iter().any(|keyword| {
                        linkedin_section
                            .text
                            .clone()
                            .unwrap_or("".to_string())
                            .to_lowercase()
                            .contains(&keyword.to_lowercase())
                    });

                    if (filter.inclusion_indicator == InclusionIndicator::IsContained
                        && !keywords_present)
                        || (filter.inclusion_indicator == InclusionIndicator::NotContained
                            && keywords_present)
                    {
                        filter_matched = false;
                        break;
                    }
                } else {
                    // If the section does not exist in the linkedin_sections, treat it as not matched
                    filter_matched = false;
                    break;
                }
            }

            if filter.condition == Condition::AND {
                if !filter_matched {
                    all_filters_matched = false;
                    break;
                }
            } else if filter.condition == Condition::OR {
                if filter_matched {
                    any_filter_matched = true;
                }
            }
        }

        if (group.filters[0].condition == Condition::AND && all_filters_matched)
            || (group.filters[0].condition == Condition::OR && any_filter_matched)
        {
            return Some(group);
        }
    }
    None
}
