use std::fmt::{Display, Formatter};
use std::io;

pub struct BidibipError {
    msg: String,
}


impl BidibipError {
    pub fn msg<T: Display>(message: T) -> Self {
        Self {
            msg: format!("{}", message),
        }
    }
}

impl From<io::Error> for BidibipError {
    fn from(value: io::Error) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<chrono::ParseError> for BidibipError {
    fn from(value: chrono::ParseError) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<reqwest::Error> for BidibipError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<serenity::prelude::SerenityError> for BidibipError {
    fn from(value: serenity::prelude::SerenityError) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<anyhow::Error> for BidibipError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<std::num::ParseIntError> for BidibipError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl From<std::string::FromUtf8Error> for BidibipError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self {
            msg: format!("{}", value),
        }
    }
}
impl Display for BidibipError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
    }
}