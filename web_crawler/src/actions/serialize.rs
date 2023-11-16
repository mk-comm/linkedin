use crate::structs::entry::{PhantomGetJson, PhantomJobs, PhantomSchools};
use crate::structs::error::CustomError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize, Serializer};
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

pub fn serialize_json(json: PhantomGetJson) -> Result<String, CustomError> {
    let jobs = json.body[0].jobs.clone();
    let schools = json.body[0].schools.clone();

    let company = match jobs {
        Some(job) => (
            extract_number(job[0].companyUrl.clone()),
            job[0].companyName.clone(),
        ),
        None => (None, None),
    };
    let education = to_education(schools)?;
    let jobs = to_experience(json.body[0].jobs.clone())?;
    let result = Profile {
        linkedin: json.body[0].general.profileUrl.clone(),
        first: json.body[0].general.firstName.clone(),
        last: json.body[0].general.lastName.clone(),
        email: None,
        job: json.job.clone(),         // should be changed
        sourcer: json.sourcer.clone(), // should be changed
        title: json.body[0].general.headline.clone(),
        linkedin_unique: extract_nick(json.body[0].general.profileUrl.clone()),
        linkedin_unique_number: json.body[0].general.userId.clone(),
        connectionLevel: Some(2),
        company: company.1,
        company_unique: company.0,
        about: json.body[0].general.description.clone(),
        profilePicture: json.body[0].general.imgUrl.clone(),
        education,
        experience: jobs,
        viewedIn: Some("Phantom".to_string()),
        location: json.body[0].general.location.clone(),
        entityUrn: json.body[0].general.vmid.clone(),
        extension_version: Some("phantom".to_string()),
        timestamp: json.body[0].timestamp.clone(),
    };
    //println!("result {:?}", result);
    let json = serde_json::to_string(&result)?;
    Ok(json)
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
        let mut start_date: Option<i64> = None;
        let mut end_date: Option<i64> = None;
        match date {
            Some(date) => {
                let dates = extract_dates(date.as_str());
                start_date = date_to_timestamp(dates[0].as_str())?;
                end_date = date_to_timestamp(dates[1].as_str())?;
                Some(dates)
            }
            None => None,
        };
        let edu = Education {
            id: extract_number(school.schoolUrl), // should be stripped of  https://www.linkedin.com/company/ and /
            schoolName: school.schoolName,
            logoUrl: school.logoUrl,
            degreeName: school.degree,
            fieldOfStudy: None,
            periodText: school.dateRange,
            startDate: start_date,
            endDate: end_date,
            description: school.description,
        };
        education.push(edu);
    }

    Ok(education)
}

fn to_experience(jobs: Option<Vec<PhantomJobs>>) -> Result<Vec<Experience>, CustomError> {
    if jobs.is_none() {
        return Ok(Vec::new());
    }
    let jobs = jobs.unwrap();
    let mut experience: Vec<Experience> = Vec::new();
    for job in jobs {
        let date = job.dateRange.clone();
        let dates: Vec<String> = match date {
            Some(date) => extract_dates(date.as_str()),
            None => {
                return Err(CustomError::ButtonNotFound(
                    "Job Date range is missing".to_string(),
                ))
            }
        };
        let start_date = date_to_timestamp(dates[0].as_str())?;
        let end_date = date_to_timestamp(dates[1].as_str())?;
        let exp = Experience {
            companyName: job.companyName.clone(),
            companyId: extract_number(job.companyUrl.clone()), //should be stripped of  https://www.linkedin.com/company/ and /
            companyURL: job.companyUrl,
            companyLetter: job.companyName.clone(),
            logo: job.logoUrl,
            position: job.jobTitle,
            employmentType: None,
            periodText: job.dateRange,
            startDate: start_date,
            endDate: end_date,
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
#[allow(deprecated)]
fn date_to_timestamp(date: &str) -> Result<Option<i64>, CustomError> {
    if date.contains("Present") {
        return Ok(None);
    }
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

    let month_year: Vec<&str> = date.split_whitespace().collect();
    let mut month = String::from(month_year[0]);

    for &(m, n) in &months {
        if m == month_year[0] {
            month = n.to_string();
        }
    }
    let date = format!("{}-{}-01", month_year[1], month);
    let naive_date = NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")?;
    let timestamp = naive_date.and_hms(0, 0, 0).timestamp();
    Ok(Some(timestamp))
}

fn extract_dates(date: &str) -> Vec<String> {
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
    let mut date_string = String::new();

    for word in date_vec {
        if word == "Present" {
            date_string = format!("{} {}", date_string, word);
            break;
        }

        for &month in &months {
            if word == month.0 {
                date_string = format!("{} {}", date_string, word);
                break;
            }
        }
        for &year in &years {
            if word == year {
                date_string = format!("{} {}", date_string, word);
            }
        }
    }
    let word_count: Vec<&str> = date_string.as_str().split_whitespace().collect();
    if word_count.len() < 3 {
        date_string = format!("Jan {} Jan {}", word_count[0], word_count[1])
    }
    let dates: Vec<String> = date_string
        .split_whitespace()
        .collect::<Vec<&str>>()
        .chunks(2)
        .map(|chunk| chunk.join(" "))
        .collect();
    dates
}
