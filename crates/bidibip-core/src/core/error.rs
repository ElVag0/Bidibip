use std::fmt::{Display, Formatter};

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