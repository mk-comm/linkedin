use crate::actions::wait::wait;
use crate::structs::entry::{PhantomGetJson, PhantomJobs, PhantomJsonProfile, PhantomSchools};
use crate::structs::error::CustomError;
#[allow(deprecated)]
use base64::encode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct ResultJson {
    b64: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct BodyJsonB64 {
    body: ResultJson,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct BodyJson {
    body: Profile,
}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Profile {
    AI: bool,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    first: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    last: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    email: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    job: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    sourcer: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    title: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    linkedin_unique_number: Option<String>,
    connectionLevel: Option<i32>,
    #[serde(serialize_with = "serialize_option_string")]
    company: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    company_unique: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    about: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    profilePicture: Option<String>,
    education: Vec<Education>,
    experience: Vec<Experience>,
    #[serde(serialize_with = "serialize_option_string")]
    viewedIn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    location: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    entityUrn: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    extension_version: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    timestamp: Option<String>,
    search_url: Option<String>
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
struct Education {
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
#[derive(Debug, Deserialize, Serialize)]
struct Experience {
    #[serde(serialize_with = "serialize_option_string")]
    companyName: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    companyId: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    companyLetter: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    companyURL: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    logo: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    position: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    employmentType: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    periodText: Option<String>,
    #[serde(serialize_with = "serialize_option_i64")]
    startDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_i64")]
    endDate: Option<i64>,
    #[serde(serialize_with = "serialize_option_string")]
    duration: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    location: Option<String>,
    #[serde(serialize_with = "serialize_option_string")]
    description: Option<String>,
}

fn serialize_option_string<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize Option<String> as a String, defaulting to "" if None
    serializer.serialize_str(value.as_ref().unwrap_or(&"".to_string()))
}

fn serialize_option_i64<S>(opt: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(val) => serializer.serialize_i64(*val),
        None => serializer.serialize_str(""),
    }
}

pub async fn serialize_json(json: PhantomGetJson) -> Result<String, CustomError> {
    //println!("result {:?}", result);
    for profile in json.body {
        let full_name = profile.general.fullName.clone();
        let result = serializer_each_profile(profile, json.job.clone(), json.sourcer.clone(), json.search_url.clone()).await;
        println!("Serilaztion result for {:?} {:?}", full_name, result);
        wait(1, 2);
    }
    Ok("test".to_owned())
    //println!("json {:?}", json);
    //Ok(())
}
fn to_education(schools: Option<Vec<PhantomSchools>>) -> Result<Vec<Education>, CustomError> {
    if schools.is_none() {
        return Ok(Vec::new());
    }
    let schools = schools.unwrap();
    let mut education: Vec<Education> = Vec::new();
    for school in schools {
        let date = school.dateRange.clone();
        //if date.is_none() {
        //return Err(CustomError::ButtonNotFound(
        //    "School Date range is missing".to_string(),
        //  ));
        //};
        let new_vec = get_date(date.as_deref());
        let date_vec = match new_vec {
            Ok(value) => value,
            Err(_) => return Err(CustomError::ButtonNotFound("Date vec is error".to_string())),
        };
        let edu = Education {
            id: extract_number(school.schoolUrl), // should be stripped of  https://www.linkedin.com/company/ and /
            schoolName: school.schoolName,
            logoUrl: school.logoUrl,
            degreeName: school.degree,
            fieldOfStudy: None,
            periodText: school.dateRange,
            startDate: date_vec[0],
            endDate: date_vec[1],
            description: school.description,
        };
        education.push(edu);
    }

    Ok(education)
}

async fn serializer_each_profile(
    json: PhantomJsonProfile,
    job: Option<String>,
    sourcer: Option<String>,
    search_url: Option<String>,
) -> Result<(), CustomError> {
    let jobs = json.jobs.clone();
    let schools = json.schools.clone();

    let company = match jobs {
        Some(job) => (
            extract_number(job[0].companyUrl.clone()),
            job[0].companyName.clone(),
        ),
        None => (None, None),
    };
    let education = to_education(schools)?;
    let jobs = to_experience(json.jobs.clone())?;
    let result = Profile {
        AI: true,
        linkedin: json.general.profileUrl.clone(),
        first: json.general.firstName.clone(),
        last: json.general.lastName.clone(),
        email: None,
        job,     // should be changed
        sourcer, // should be changed
        title: json.general.headline.clone(),
        linkedin_unique: extract_nick(json.general.profileUrl.clone()),
        linkedin_unique_number: json.general.userId.clone(),
        connectionLevel: Some(2),
        company: company.1,
        company_unique: company.0,
        about: json.general.description.clone(),
        profilePicture: json.general.imgUrl.clone(),
        education,
        experience: jobs,
        viewedIn: Some("Phantom".to_string()),
        location: json.general.location.clone(),
        entityUrn: json.general.vmid.clone(),
        extension_version: Some("phantom".to_string()),
        timestamp: json.timestamp.clone(),
        search_url
    };
    let result = send_url(result).await;
    match result {
        Ok(_) => (),
        Err(e) => println!("result error {}", e),
    }
    Ok(())
}
#[allow(deprecated)]
async fn send_url(profile: Profile) -> Result<(), CustomError> {
    let serialized = serde_json::to_vec(&profile).unwrap();
    let encoded = encode(&serialized);
    const WEBHOOK_URL: &str = "https://overview.tribe.xyz/api/1.1/wf/chromedata_view";
    let client = reqwest::Client::new();

    let target_json = json!({ 
        "b64": encoded });
    let res = client.post(WEBHOOK_URL).json(&target_json).send().await;
    match res {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }
    Ok(())
}

fn to_experience(jobs: Option<Vec<PhantomJobs>>) -> Result<Vec<Experience>, CustomError> {
    if jobs.is_none() {
        return Ok(Vec::new());
    }
    let jobs = jobs.unwrap();
    let mut experience: Vec<Experience> = Vec::new();
    for job in jobs {
        let date = job.dateRange.clone();
        //if date.is_none() {
        //  return Err(CustomError::ButtonNotFound(
        //    "Job Date range is missing".to_string(),
        //));
        //};
        let new_vec = get_date(date.as_deref());
        let date_vec = match new_vec {
            Ok(value) => value,
            Err(_) => return Err(CustomError::ButtonNotFound("Date vec is error".to_string())),
        };
        let exp = Experience {
            companyName: job.companyName.clone(),
            companyId: extract_number(job.companyUrl.clone()), //should be stripped of  https://www.linkedin.com/company/ and /
            companyURL: job.companyUrl,
            companyLetter: job.companyName.clone(),
            logo: job.logoUrl,
            position: job.jobTitle,
            employmentType: None,
            periodText: job.dateRange,
            startDate: date_vec[0],
            endDate: date_vec[1],
            duration: job.duration,
            location: job.location,
            description: job.description,
        };
        experience.push(exp);
    }
    Ok(experience)
}

fn extract_number(url: Option<String>) -> Option<String> {
    url.and_then(|url| {
        let trimmed_url = url.trim_end_matches('/');
        trimmed_url
            .split('/')
            .filter(|s| s.chars().all(char::is_numeric))
            .last()
            .map(|s| s.to_string())
    })
}
fn extract_nick(url: Option<String>) -> Option<String> {
    url.and_then(|url| {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        parts.last().map(|s| s.to_string())
    })
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
fn get_date(date: Option<&str>) -> Result<Vec<Option<i64>>, CustomError> {
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
