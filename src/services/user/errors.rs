use std::error::Error;
use std::fmt;

use postgres::Error as PostgresError;

#[derive(Debug)]
pub enum ServiceError {
    Default(String),
    NotFound(Option<String>),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ServiceError::Default(ref msg) => write!(f, "User service error: {}", msg),
            ServiceError::NotFound(ref opt) => match opt {
                Some(msg) => write!(f, "User not found: {}", msg),
                None => write!(f, "User not found"),
            },
        }
    }
}

impl Error for ServiceError {}

impl From<PostgresError> for ServiceError {
    fn from(pg_err: PostgresError) -> Self {
        Self::Default(pg_err.to_string())
    }
}
