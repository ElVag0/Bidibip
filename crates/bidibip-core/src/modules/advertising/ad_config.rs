use serde::{Deserialize, Serialize};
use crate::core::utilities::Username;

#[derive(Serialize, Deserialize)]
pub struct Ad {
    title: String,
    studio: Option<String>,
    remote: IsRemote,
    compensation: Compensation,
    description: String,
    responsibilities: String,
    qualifications: String,
    apply_at: ApplyAt,
    other_urls: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub enum ApplyAt {
    Discord(Username),
    Other(String)
}


#[derive(Serialize, Deserialize)]
pub enum IsRemote {
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
pub enum AdInfo {
    Volunteering,
    Internship,
    Freelance,
    WorkStudy,
    FixedTerm(FixedTermInfos), // CDD
    OpenEnded(OpenEndedInfos) // CDI
}
