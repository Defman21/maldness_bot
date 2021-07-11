use crate::errors::HandleUpdateError;
use crate::services::user::errors::ServiceError as UserServiceError;
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
            ServiceError::Default(ref msg) => write!(f, "Sleep events service error: {}", msg),
            ServiceError::NotFound => write!(f, "Sleep event not found"),
        }
    }
}

impl Error for ServiceError {}

impl From<DieselError> for ServiceError {
    fn from(pg_err: DieselError) -> Self {
        Self::Default(pg_err.to_string())
    }
}

impl From<ServiceError> for HandleUpdateError {
    fn from(err: ServiceError) -> Self {
        Self::Command(err.to_string())
    }
}

impl From<UserServiceError> for ServiceError {
    fn from(err: UserServiceError) -> Self {
        Self::Default(err.to_string())
    }
}
