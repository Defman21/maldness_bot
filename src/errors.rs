use std::error::Error;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub enum HandleUpdateError {
    Command(String),
    Skip,
    Api(frankenstein::Error),
}

impl fmt::Display for HandleUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HandleUpdateError::Command(ref err) => write!(f, "Command error: {}", err),
            HandleUpdateError::Api(ref err) => match err {
                frankenstein::Error::HttpError(err) => {
                    write!(f, "HTTP error: {} {}", err.code, err.message)
                }
                frankenstein::Error::ApiError(err) => {
                    write!(f, "API error: {} {}", err.error_code, err.description)
                }
            },
            HandleUpdateError::Skip => write!(f, "Update skipped"),
        }
    }
}

impl Error for HandleUpdateError {}

impl From<frankenstein::Error> for HandleUpdateError {
    fn from(err: frankenstein::Error) -> Self {
        Self::Api(err)
    }
}
