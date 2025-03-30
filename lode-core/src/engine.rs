use crate::error::Result;
use crate::http::HttpClient;
use crate::metrics::{RequestMetrics, TestMetrics};

use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use reqwest::Method;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, instrument, warn};

/// Load test engine that executes HTTP requests concurrently
pub struct LoadTestEngine<C: HttpClient> {
    client: Arc<C>,
}

impl<C: HttpClient> LoadTestEngine<C> {
    /// Create a new load test engine with the given HTTP client
    #[instrument(skip(client))]
    pub fn new(client: C) -> Result<Self> {
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Run the load test with the given parameters
    #[instrument(skip(self, headers, body, progress_bar), fields(
        method = %method,
        url = %url,
        num_requests = %num_requests,
        concurrency = %concurrency,
        timeout_ms = %timeout.as_millis(),
    ))]
    pub async fn run(
        &self,
        method: Method,
        url: String,
        num_requests: u64,
        concurrency: u64,
        timeout: Duration,
        headers: Vec<(String, String)>,
        body: Option<String>,
        progress_bar: Option<ProgressBar>,
    ) -> Result<Arc<Mutex<TestMetrics>>> {
        info!(
            "Starting load test:\n\
             Target: {} {}\n\
             Requests: {}\n\
             Concurrency: {}\n\
             Timeout: {:?}",
            method, url, num_requests, concurrency, timeout
        );

        let start_time = std::time::Instant::now();
        let metrics = Arc::new(Mutex::new(TestMetrics::new()?));
        let metrics_for_stream = Arc::clone(&metrics);

        stream::iter((0..num_requests).map(move |i| {
            let client = Arc::clone(&self.client);
            let metrics = Arc::clone(&metrics_for_stream);
            let url = url.clone();
            let method = method.clone();
            let headers = headers.clone();
            let body = body.clone();
            let progress_bar = progress_bar.clone();

            let span = tracing::info_span!(
                "request",
                request_id = %i,
                method = %method,
                url = %url
            );

            async move {
                let _enter = span.enter();
                let request_metrics = RequestMetrics::new();
                let result = client
                    .send_request(method, url, timeout, headers, body)
                    .await;

                let mut metrics = metrics.lock().await;
                match result {
                    Ok(response) => {
                        metrics.record_request(request_metrics.complete(response.status()));
                    }
                    Err(error) => {
                        metrics.record_request(request_metrics.record_error(error));
                    }
                }

                if let Some(pb) = progress_bar {
                    pb.inc(1);
                }
            }
        }))
        .buffer_unordered(concurrency as usize)
        .collect::<Vec<_>>()
        .await;

        let duration = start_time.elapsed();
        {
            let mut metrics = metrics.lock().await;
            metrics.finalize(duration).await?;
        }

        Ok(metrics)
    }
}
