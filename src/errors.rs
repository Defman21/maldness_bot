use crate::services::weather::WeatherError;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub enum HandleUpdateError {
    Command(String),
    Skip,
    Api(frankenstein::Error),
    Service(Box<dyn Error>),
    NotAllowed {
        chat_id: i64,
        reason: String,
        chat_name: String,
        chat_type: String,
    },
}

impl fmt::Display for HandleUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Command(ref err) => write!(f, "Command error: {}", err),
            Self::Api(ref err) => match err {
                frankenstein::Error::HttpError(err) => {
                    write!(f, "HTTP error: {} {}", err.code, err.message)
                }
                frankenstein::Error::ApiError(err) => {
                    write!(f, "API error: {} {}", err.error_code, err.description)
                }
            },
            Self::Skip => write!(f, "Update skipped"),
            Self::Service(ref err) => write!(f, "Service error: {}", err),
            Self::NotAllowed {
                ref chat_id,
                ref reason,
                ref chat_name,
                ref chat_type,
            } => write!(
                f,
                "Chat ({}) not allowed (disallowed by {}): {} ({})",
                chat_type, reason, chat_id, chat_name,
            ),
        }
    }
}

impl Error for HandleUpdateError {}

impl From<frankenstein::Error> for HandleUpdateError {
    fn from(err: frankenstein::Error) -> Self {
        Self::Api(err)
    }
}

impl From<WeatherError> for HandleUpdateError {
    fn from(err: WeatherError) -> Self {
        Self::Service(Box::new(err))
    }
}
