use diesel::result::Error as DieselError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    Default(String),
    NotFound,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ServiceError::Default(ref msg) => write!(f, "User service error: {}", msg),
            ServiceError::NotFound => write!(f, "User not found"),
        }
    }
}

impl Error for ServiceError {}

impl From<DieselError> for ServiceError {
    fn from(pg_err: DieselError) -> Self {
        Self::Default(pg_err.to_string())
    }
}
