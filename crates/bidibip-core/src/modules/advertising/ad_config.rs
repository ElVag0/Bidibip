use serde::{Deserialize, Serialize};
use crate::core::utilities::Username;

#[derive(Serialize, Deserialize, Default)]
pub struct AdDescription {
    pub is_searching: Option<bool>,
    pub kind: Option<Contract>,
    pub location: Option<Location>,
    pub studio: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub responsibilities: Option<String>,
    pub qualifications: Option<String>,
    pub apply_at: Option<ApplyAt>,
    pub other_urls: Option<Vec<String>>
}

#[derive(Serialize, Deserialize)]
pub enum ApplyAt {
    Discord(Username),
    Other(String)
}


#[derive(Serialize, Deserialize)]
pub enum Location {
    Remote,
    Unspecified,
    OnSiteFlex(String),
    OnSite(String),
}

#[derive(Serialize, Deserialize)]
pub struct FixedTermInfos {
    duration: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenEndedInfos {
}

#[derive(Serialize, Deserialize)]
pub enum Compensation {
    Free,
    Paid(String)
}

#[derive(Serialize, Deserialize)]
pub enum Contract {
    Volunteering,
    Internship(bool), // paid or not
    Freelance,
    WorkStudy,
    FixedTerm(FixedTermInfos), // CDD
    OpenEnded(OpenEndedInfos) // CDI
}
