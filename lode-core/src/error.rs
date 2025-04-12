/// Main error type for the load testing library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Metrics error: {0}")]
    Metrics(String),

    #[error("Report error: {0}")]
    Report(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Metrics(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Http(error.to_string())
    }
}

/// Result type alias for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_from_string() {
        let error_msg = "test error".to_string();
        let error = Error::from(error_msg.clone());
        assert!(matches!(error, Error::Metrics(msg) if msg == error_msg));
    }

    #[test]
    fn test_error_from_reqwest() {
        let client = reqwest::Client::new();
        let reqwest_error = client.get("not-a-url").build().unwrap_err();
        let error = Error::from(reqwest_error);
        assert!(matches!(error, Error::Http(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();
        assert!(matches!(error, Error::Io(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: Error = json_error.into();
        assert!(matches!(error, Error::Json(_)));
    }

    #[test]
    fn test_error_variants() {
        let config_error = Error::Config("test config error".to_string());
        assert!(matches!(config_error, Error::Config(_)));

        let http_error = Error::Http("test http error".to_string());
        assert!(matches!(http_error, Error::Http(_)));

        let metrics_error = Error::Metrics("test metrics error".to_string());
        assert!(matches!(metrics_error, Error::Metrics(_)));

        let report_error = Error::Report("test report error".to_string());
        assert!(matches!(report_error, Error::Report(_)));
    }
}
