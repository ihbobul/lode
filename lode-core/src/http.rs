use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Method, Response};
use std::time::Duration;
use tracing::{instrument, warn};

/// HTTP client trait for making requests
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request and return the raw response or error
    async fn send_request(
        &self,
        method: Method,
        url: String,
        timeout: Duration,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<Response, ReqwestError>;
}

/// Default HTTP client implementation using reqwest
pub struct DefaultHttpClient {
    client: Client,
}

impl DefaultHttpClient {
    /// Create a new default HTTP client
    #[instrument(skip_all)]
    pub fn new() -> Result<Self, ReqwestError> {
        Ok(Self {
            client: Client::new(),
        })
    }
}

#[async_trait]
impl HttpClient for DefaultHttpClient {
    #[instrument(skip(self, headers, body), fields(
        method = %method,
        url = %url,
        timeout_ms = %timeout.as_millis(),
        num_headers = %headers.len(),
        has_body = %body.is_some(),
    ))]
    async fn send_request(
        &self,
        method: Method,
        url: String,
        timeout: Duration,
        headers: Vec<(String, String)>,
        body: Option<String>,
    ) -> Result<Response, ReqwestError> {
        let mut request = self.client.request(method, url).timeout(timeout);

        for (name, value) in headers {
            request = request.header(name, value);
        }

        if let Some(body) = body {
            request = request.body(body);
        }

        match request.send().await {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("Request failed: {}", e);
                Err(e)
            }
        }
    }
}

impl Default for DefaultHttpClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}
