use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid concurrency: {0}")]
    InvalidConcurrency(String),
    #[error("Invalid number of requests: {0}")]
    InvalidRequests(String),
    #[error("Invalid timeout: {0}")]
    InvalidTimeout(String),
    #[error("Invalid method: {0}")]
    InvalidMethod(String),
}

/// HTTP methods supported by the load tester
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        }
    }
}

impl FromStr for HttpMethod {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            _ => Err(ConfigError::InvalidMethod(s.to_string())),
        }
    }
}

/// Configuration for a load test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoadTestConfig {
    /// Target URL to test
    pub url: String,

    /// HTTP method to use
    pub method: HttpMethod,

    /// Number of total requests to make
    pub requests: usize,

    /// Number of concurrent requests
    pub concurrency: usize,

    /// Request timeout in seconds
    pub timeout: Duration,

    /// Optional request headers
    pub headers: Vec<(String, String)>,

    /// Optional request body
    pub body: Option<String>,
}

impl LoadTestConfig {
    /// Create a new load test configuration
    pub fn new(
        url: String,
        method: HttpMethod,
        requests: usize,
        concurrency: usize,
        timeout: Duration,
    ) -> Result<Self, ConfigError> {
        if let Err(e) = Url::parse(&url) {
            return Err(ConfigError::InvalidUrl(e.to_string()));
        }

        if requests == 0 {
            return Err(ConfigError::InvalidRequests(
                "Number of requests must be greater than 0".to_string(),
            ));
        }

        if concurrency == 0 {
            return Err(ConfigError::InvalidConcurrency(
                "Concurrency must be greater than 0".to_string(),
            ));
        }
        if concurrency > requests {
            return Err(ConfigError::InvalidConcurrency(
                "Concurrency cannot be greater than the number of requests".to_string(),
            ));
        }

        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Timeout must be greater than 0 seconds".to_string(),
            ));
        }

        Ok(Self {
            url,
            method,
            requests,
            concurrency,
            timeout,
            headers: Vec::new(),
            body: None,
        })
    }

    /// Add a header to the configuration
    pub fn with_header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    /// Add a body to the configuration
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::POST);
        assert_eq!(HttpMethod::from_str("PUT").unwrap(), HttpMethod::PUT);
        assert_eq!(HttpMethod::from_str("DELETE").unwrap(), HttpMethod::DELETE);
        assert_eq!(HttpMethod::from_str("PATCH").unwrap(), HttpMethod::PATCH);

        assert!(matches!(
            HttpMethod::from_str("INVALID").unwrap_err(),
            ConfigError::InvalidMethod(_)
        ));
    }

    #[test]
    fn test_http_method_into_reqwest() {
        assert_eq!(reqwest::Method::from(HttpMethod::GET), reqwest::Method::GET);
        assert_eq!(
            reqwest::Method::from(HttpMethod::POST),
            reqwest::Method::POST
        );
        assert_eq!(reqwest::Method::from(HttpMethod::PUT), reqwest::Method::PUT);
        assert_eq!(
            reqwest::Method::from(HttpMethod::DELETE),
            reqwest::Method::DELETE
        );
        assert_eq!(
            reqwest::Method::from(HttpMethod::PATCH),
            reqwest::Method::PATCH
        );
    }

    #[test]
    fn test_load_test_config_validation() {
        // Test invalid URL
        assert!(matches!(
            LoadTestConfig::new(
                "invalid-url".to_string(),
                HttpMethod::GET,
                100,
                10,
                Duration::from_secs(5)
            )
            .unwrap_err(),
            ConfigError::InvalidUrl(_)
        ));

        // Test zero concurrency
        assert!(matches!(
            LoadTestConfig::new(
                "http://example.com".to_string(),
                HttpMethod::GET,
                100,
                0,
                Duration::from_secs(5)
            )
            .unwrap_err(),
            ConfigError::InvalidConcurrency(_)
        ));

        // Test concurrency greater than requests
        assert!(matches!(
            LoadTestConfig::new(
                "http://example.com".to_string(),
                HttpMethod::GET,
                10,
                20,
                Duration::from_secs(5)
            )
            .unwrap_err(),
            ConfigError::InvalidConcurrency(_)
        ));

        // Test zero requests
        assert!(matches!(
            LoadTestConfig::new(
                "http://example.com".to_string(),
                HttpMethod::GET,
                0,
                10,
                Duration::from_secs(5)
            )
            .unwrap_err(),
            ConfigError::InvalidRequests(_)
        ));

        // Test zero timeout
        assert!(matches!(
            LoadTestConfig::new(
                "http://example.com".to_string(),
                HttpMethod::GET,
                100,
                10,
                Duration::from_secs(0)
            )
            .unwrap_err(),
            ConfigError::InvalidTimeout(_)
        ));
    }

    #[test]
    fn test_load_test_config_with_header_and_body() {
        let config = LoadTestConfig::new(
            "http://example.com".to_string(),
            HttpMethod::POST,
            100,
            10,
            Duration::from_secs(5),
        )
        .unwrap();

        let config = config
            .with_header("Content-Type".to_string(), "application/json".to_string())
            .with_body(r#"{"test": "data"}"#.to_string());

        assert_eq!(config.headers.len(), 1);
        assert_eq!(config.headers[0].0, "Content-Type");
        assert_eq!(config.headers[0].1, "application/json");
        assert_eq!(config.body, Some(r#"{"test": "data"}"#.to_string()));
    }
}
