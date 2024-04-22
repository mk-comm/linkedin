use crate::structs::error::CustomError;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};
pub fn serialize_option_string<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize Option<String> as a String, defaulting to "" if None
    serializer.serialize_str(value.as_ref().unwrap_or(&"".to_string()))
}

pub fn serialize_option_i64<S>(opt: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(val) => serializer.serialize_i64(*val),
        None => serializer.serialize_str(""),
    }
}
pub fn split_around_dot(input: &str) -> (Option<String>, Option<String>) {
    let delimiter = " · ";
    if let Some(index) = input.find(delimiter) {
        let before = input[..index].trim().to_string();
        let after = input[index + delimiter.len()..].trim().to_string();
        (Some(before), Some(after))
    } else {
        (None, None) // Return None if delimiter is not found
    }
}

pub fn split_around_comma(input: &str) -> (Option<String>, Option<String>) {
    let delimiter = ", ";
    if let Some(index) = input.find(delimiter) {
        let before = input[..index].trim().to_string();
        let after = input[index + delimiter.len()..].trim().to_string();
        (Some(before), Some(after))
    } else {
        (None, None) // Return None if delimiter is not found
    }
}
pub fn get_date(date: Option<&str>) -> Result<Vec<Option<i64>>, CustomError> {
    if date.is_none() {
        let new_vec: Vec<Option<i64>> = vec![None, None];
        return Ok(new_vec);
    }
    let date = date.unwrap();
    let date_vec: Vec<&str> = date.split_whitespace().collect();

    let months = [
        ("Jan", 1),
        ("Feb", 2),
        ("Mar", 3),
        ("Apr", 4),
        ("May", 5),
        ("Jun", 6),
        ("Jul", 7),
        ("Aug", 8),
        ("Sep", 9),
        ("Oct", 10),
        ("Nov", 11),
        ("Dec", 12),
    ];
    let year_strings: Vec<String> = (1940..=2090).map(|year| year.to_string()).collect();
    let years: Vec<&str> = year_strings.iter().map(AsRef::as_ref).collect();
    let mut start_date = DateFormat::new();
    let mut end_date = DateFormat::new();
    for word in date_vec {
        if word == "Present" {
            end_date.month = None;
            end_date.year = None;
            break;
        }

        for &month in &months {
            if word == month.0 {
                if start_date.month.is_none() {
                    start_date.month = Some(month.1.to_string())
                } else {
                    end_date.month = Some(month.1.to_string())
                };
                break;
            }
        }
        for &year in &years {
            if word == year {
                if start_date.year.is_none() {
                    start_date.year = Some(word.to_string())
                } else {
                    end_date.year = Some(word.to_string())
                };
            }
        }
    }
    let start_date_string = construct_string(start_date);
    let end_date_string = construct_string(end_date);
    let start_date_i64 = match construct_timestamp(start_date_string) {
        Ok(value) => value,
        Err(e) => {
            return Err(CustomError::ButtonNotFound(
                format!("start_date_i64_error {}", e).to_string(),
            ));
        }
    };
    let end_date_i64 = match construct_timestamp(end_date_string) {
        Ok(value) => value,
        Err(e) => {
            return Err(CustomError::ButtonNotFound(
                format!("end_date_i64_error {}", e).to_string(),
            ));
        }
    };
    let vec_i64 = vec![start_date_i64, end_date_i64];
    Ok(vec_i64)
}

#[allow(deprecated)]
fn construct_timestamp(date: Option<String>) -> Result<Option<i64>, CustomError> {
    if date.is_none() {
        return Ok(None);
    }
    let naive_date = NaiveDate::parse_from_str(date.unwrap().as_str(), "%Y-%m-%d")?;
    let timestamp = naive_date.and_hms(0, 0, 0).timestamp();
    Ok(Some(timestamp))
}

fn construct_string(date: DateFormat) -> Option<String> {
    //println!("check {:?}", date);
    if date.month.is_none() && date.year.is_none() {
        return None;
    };
    //println!("checkafter {:?}", date);

    let month = if date.month.is_some() {
        date.month
    } else {
        Some("1".to_string())
    };

    let year = if date.year.is_some() {
        date.year
    } else {
        return None;
    };

    let result = format!("{}-{}-01", year.unwrap(), month.unwrap());
    Some(result)
}
#[derive(Debug)]
struct DateFormat {
    month: Option<String>,
    year: Option<String>,
}

impl DateFormat {
    fn new() -> Self {
        DateFormat {
            month: None,
            year: None,
        }
    }
}
