use crate::structs::entry::{Condition, EvaluateProfile, Filter};

pub async fn eval(entry: EvaluateProfile) -> bool {
    let filters = entry.filters.clone();
    let mut list = update_list_filter(filters);
    sort_filters(&mut list);
    let not_fit = run_evaluation(list, entry.reasons);
    not_fit
}

fn update_list_filter(list: Vec<Filter>) -> Vec<Filter> {
    let mut new_list: Vec<Filter> = Vec::new();
    for mut filter in list {
        new_list.push(filter.update_filter());
        println!("{:?}", filter);
    }
    new_list
}

fn run_evaluation(filters: Vec<Filter>, reasons: Vec<String>) -> bool {
    if reasons.is_empty() {
        return false;
    }
    for filter in filters {
        println!("loop {:?}", filter);
        match filter.condition {
            Condition::AND => {
                if reasons.contains(&filter.response_key.unwrap()) {
                    return true;
                }
            }
            Condition::OR => {}
        }
    }
    true
}

fn sort_filters(filters: &mut Vec<Filter>) {
    filters.sort_by(|a, b| {
        let rank_a = filter_priority(&a.filter, &a.condition);
        let rank_b = filter_priority(&b.filter, &b.condition);
        rank_a.cmp(&rank_b)
    });
}

fn filter_priority(filter_name: &str, condition: &Condition) -> u8 {
    match (filter_name, condition) {
        ("Total years of experience", Condition::AND) => 1,
        ("Role", Condition::AND) => 2,
        ("Skill", Condition::OR) => 3,
        ("Recent job", Condition::AND) => 4,
        ("Recent skills", Condition::AND) => 5,
        ("Months in current position", Condition::AND) => 6,
        // Default case if the filter is not recognized
        _ => u8::MAX,
    }
}
