use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AdDescription {
    pub is_searching: Option<bool>,
    pub kind: Option<Contract>,
    pub location: Option<Location>,
    pub studio: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub responsibilities: Option<String>,
    pub qualifications: Option<String>,
    pub contact: Option<Contact>,
    pub other_urls: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Contact {
    Discord,
    Other(Option<String>)
}


#[derive(Serialize, Deserialize, Clone)]
pub enum Location {
    Remote,
    OnSiteFlex(Option<String>),
    OnSite(Option<String>),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct InternshipInfos {
    pub duration: Option<String>,
    pub compensation: Option<Option<String>>, // Paid or not
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FreelanceInfos {
    pub duration: Option<String>,
    pub compensation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorkStudyInfos {
    pub duration: Option<String>,
    pub compensation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FixedTermInfos {
    pub duration: Option<String>,
    pub compensation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OpenEndedInfos {
    pub compensation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Contract {
    Volunteering,
    Internship(InternshipInfos), // paid or not
    Freelance(FreelanceInfos),
    WorkStudy(WorkStudyInfos),
    FixedTerm(FixedTermInfos), // CDD
    OpenEnded(OpenEndedInfos) // CDI
}
