use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluateProfile {
    pub profile_id: String,
    pub filters: Vec<Filter>,
    pub reasons: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Filter {
    pub filter: String,
    pub condition: Condition,
    pub response_key: Option<String>,
}
#[derive(Debug)]
pub struct FilterGroup {
    filters: Vec<Filter>,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Condition {
    AND,
    OR,
}

impl Filter {
    pub fn update_filter(&mut self) -> Filter {
        let response_key = match self.filter.as_str() {
            "Total years of experience" => "Experience level",
            "Job title" => "Job title",
            "Skill" => "Skill level",
            "Recent job title" => "Type of experience",
            "Recent skill" => "Type of skill/s",
            "Keyword" => "Keyword",
            "Months in current position" => "Time in current position",
            _ => "Invalid filter type",
        };
        Filter {
            filter: self.filter.clone(),
            condition: self.condition.clone(),
            response_key: Some(response_key.to_string()),
        }
    }
}

pub async fn eval(entry: EvaluateProfile) -> bool {
    let filters = entry.filters.clone();
    let mut list = update_list_filter(filters);
    sort_filters(&mut list);
    let group = group_filters(list);
    println!("groupss: {:?}", group);
    run_evaluation(group, entry.reasons)
}

fn update_list_filter(list: Vec<Filter>) -> Vec<Filter> {
    let mut new_list: Vec<Filter> = Vec::new();
    for mut filter in list {
        new_list.push(filter.update_filter());
        println!("{:?}", filter);
    }
    new_list
}
fn group_filters(filters: Vec<Filter>) -> Vec<FilterGroup> {
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
fn run_evaluation(groups: Vec<FilterGroup>, reasons: Vec<String>) -> bool {
    for (i, group) in groups.iter().enumerate() {
        let mut group_matched = false;

        for filter in &group.filters {
            let response_key = filter.response_key.as_ref().unwrap();

            if reasons.contains(response_key) {
                group_matched = true;
                break;
            }
        }

        if group_matched {
            if i == groups.len() - 1 {
                return false;
            }
        } else {
            if i != groups.len() - 1 {
                return true;
            }
        }
    }
    true
}

fn sort_filters(filters: &mut Vec<Filter>) {
    filters.sort_by(|a, b| {
        let rank_a = filter_priority(&a.filter);
        let rank_b = filter_priority(&b.filter);
        rank_a.cmp(&rank_b)
    });
}

fn filter_priority(filter_name: &str) -> u8 {
    match (filter_name) {
        "Total years of experience" => 1,
        "Job title" => 2,
        "Skill" => 3,
        "Recent job title" => 4,
        "Recent skill" => 5,
        "Keyword" => 6,
        "Months in current position" => 7,
        // Default case if the filter is not recognized
        _ => u8::MAX,
    }
}
