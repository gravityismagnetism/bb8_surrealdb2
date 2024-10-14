use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct ConnectionError {
    pub error: DatabaseConnectionErrors,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ConnectionError: {}", self.error)
    }
}

impl Error for ConnectionError {
fn description(&self) -> &str {
    self.error.as_str()
}
}

#[derive(Debug)]
pub enum DatabaseConnectionErrors {
    Default,
    InvalidNamespace,
    InvalidDatabaseName,
    InvalidHost,
    InvalidPort,
    InvalidUsername,
    InvalidPassword,
    HealthCheckFailed,
    NotYetImplemented,
    PoolConnectionError,
}

impl Display for DatabaseConnectionErrors {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "DatabaseConnectionError: {}", self)
    }
}

impl DatabaseConnectionErrors {
    pub fn as_str(&self) -> &str {
        self.into()
    }
}

impl From<&DatabaseConnectionErrors> for &str {
    fn from(error: &DatabaseConnectionErrors) -> Self {
        match error {
            DatabaseConnectionErrors::Default => "Database error",
            DatabaseConnectionErrors::InvalidNamespace => "Namespace not set error",
            DatabaseConnectionErrors::InvalidDatabaseName => "Database Name not set error",
            DatabaseConnectionErrors::InvalidHost => "Host not set error",
            DatabaseConnectionErrors::InvalidPort => {
                "Port not set error: Port must be greater than 1024"
            }
            DatabaseConnectionErrors::InvalidUsername => "Username not set error",
            DatabaseConnectionErrors::InvalidPassword => "Password not set error",
            DatabaseConnectionErrors::HealthCheckFailed => "Health check failed error",
            DatabaseConnectionErrors::NotYetImplemented => {
                "Connection type not yet implemented error"
            }
            DatabaseConnectionErrors::PoolConnectionError => "Pool connection error",
        }
    }
}

impl Error for DatabaseConnectionErrors {
    fn description(&self) -> &str {
        self.as_str()
    }
}
