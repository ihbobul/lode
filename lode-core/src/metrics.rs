use hdrhistogram::Histogram;
use reqwest::{Error as ReqwestError, StatusCode};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, instrument, warn};

/// Metrics for a single request
#[derive(Debug)]
pub struct RequestMetrics {
    start_time: Instant,
    duration: Option<Duration>,
    status: Option<StatusCode>,
    error: Option<ReqwestError>,
}

impl RequestMetrics {
    /// Create new request metrics
    #[instrument(skip_all)]
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            duration: None,
            status: None,
            error: None,
        }
    }

    /// Complete the request with a status code
    #[instrument(skip(self))]
    pub fn complete(mut self, status: StatusCode) -> Self {
        if self.duration.is_none() {
            self.duration = Some(self.start_time.elapsed());
        }
        self.status = Some(status);
        self
    }

    /// Record a reqwest error
    #[instrument(skip(self))]
    pub fn record_error(mut self, error: ReqwestError) -> Self {
        if self.duration.is_none() {
            self.duration = Some(self.start_time.elapsed());
        }
        self.error = Some(error);
        self
    }

    /// Get the start time
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Get the duration
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Get the status code
    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }

    /// Get the error
    pub fn error(&self) -> Option<&ReqwestError> {
        self.error.as_ref()
    }
}

/// Metrics for a load test
#[derive(Debug)]
pub struct TestMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_duration: Duration,
    requests_per_second: f64,
    response_times: Histogram<u64>,
    error_counts: HashMap<String, u64>,
    error_messages: Vec<String>,
    log_batch_size: u64,
    last_batch_log: Instant,
}

impl TestMetrics {
    /// Create new test metrics
    #[instrument(skip_all)]
    pub fn new() -> Result<Self, String> {
        info!("Creating new test metrics");
        Ok(Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::from_secs(0),
            requests_per_second: 0.0,
            response_times: Histogram::new_with_bounds(1, 60_000_000, 3)
                .map_err(|e| e.to_string())?,
            error_counts: HashMap::new(),
            error_messages: Vec::new(),
            log_batch_size: 100,
            last_batch_log: Instant::now(),
        })
    }

    /// Record a request
    #[instrument(skip(self, metrics))]
    pub fn record_request(&mut self, metrics: RequestMetrics) {
        self.total_requests += 1;

        if let Some(duration) = metrics.duration {
            if duration.as_micros() >= 1 {
                let duration_us = duration.as_micros() as u64;
                let _ = self.response_times.record(duration_us);
            }
        }

        match (metrics.status(), metrics.error()) {
            (Some(status), None) => {
                if status.is_success() {
                    self.successful_requests += 1;
                } else {
                    self.failed_requests += 1;
                    let error_type = format!("HTTP {}", status.as_u16());
                    *self.error_counts.entry(error_type.clone()).or_insert(0) += 1;
                    let error_message = status
                        .canonical_reason()
                        .unwrap_or(&status.to_string())
                        .to_string();
                    self.error_messages.push(error_message);
                }
            }
            (None, Some(error)) => {
                self.failed_requests += 1;
                let error_type = error
                    .status()
                    .map(|s| format!("HTTP {}", s.as_u16()))
                    .unwrap_or_else(|| "Unknown Error".to_string());
                *self.error_counts.entry(error_type.clone()).or_insert(0) += 1;
                let error_message = error.to_string();
                self.error_messages.push(error_message);
            }
            (None, None) => {
                self.failed_requests += 1;
                let error_type = "Unknown Error".to_string();
                *self.error_counts.entry(error_type.clone()).or_insert(0) += 1;
                self.error_messages.push(error_type);
            }
            (Some(_), Some(_)) => {
                self.failed_requests += 1;
                let error_type = "Unknown Error".to_string();
                *self.error_counts.entry(error_type.clone()).or_insert(0) += 1;
                self.error_messages.push(error_type);
            }
        }

        if self.total_requests % self.log_batch_size == 0 {
            let elapsed = self.last_batch_log.elapsed();
            let current_rps = self.log_batch_size as f64 / elapsed.as_secs_f64();
            let success_rate =
                (self.successful_requests as f64 / self.total_requests as f64) * 100.0;
            info!(
                "Progress: {}/{} requests ({:.1}%)\n\
                 Current RPS: {:.2}\n\
                 Success Rate: {:.1}% ({}/{})\n\
                 Error Rate: {:.1}% ({}/{})",
                self.total_requests,
                self.total_requests,
                (self.total_requests as f64 / self.total_requests as f64) * 100.0,
                current_rps,
                success_rate,
                self.successful_requests,
                self.total_requests,
                100.0 - success_rate,
                self.failed_requests,
                self.total_requests
            );
            self.last_batch_log = Instant::now();
        }
    }

    /// Finalize the metrics with the total duration
    #[instrument(skip(self))]
    pub async fn finalize(&mut self, duration: Duration) -> Result<(), String> {
        info!("Finalizing test metrics");
        self.total_duration = duration;
        self.requests_per_second = if duration.as_secs_f64() > 0.0 {
            self.total_requests as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        let success_rate = (self.successful_requests as f64 / self.total_requests as f64) * 100.0;
        let error_rate = (self.failed_requests as f64 / self.total_requests as f64) * 100.0;

        info!(
            "Load Test Results:\n\
             Duration: {:?}\n\
             Total Requests: {}\n\
             Average RPS: {:.2}\n\
             Success Rate: {:.1}% ({}/{})\n\
             Error Rate: {:.1}% ({}/{})\n\
             Response Times:\n\
             - Min: {:?}\n\
             - Max: {:?}\n\
             - Mean: {:?}\n\
             - Median: {:?}\n\
             - P95: {:?}\n\
             - P99: {:?}",
            duration,
            self.total_requests,
            self.requests_per_second,
            success_rate,
            self.successful_requests,
            self.total_requests,
            error_rate,
            self.failed_requests,
            self.total_requests,
            self.min_response_time(),
            self.max_response_time(),
            self.mean_response_time(),
            self.median_response_time(),
            self.p95_response_time(),
            self.p99_response_time()
        );

        if !self.error_counts.is_empty() {
            info!("Error Distribution:");
            for (error_type, count) in &self.error_counts {
                info!("  {}: {} occurrences", error_type, count);
            }
        }

        Ok(())
    }

    /// Get total number of requests
    pub fn total_requests(&self) -> u64 {
        self.total_requests
    }

    /// Get number of successful requests
    pub fn successful_requests(&self) -> u64 {
        self.successful_requests
    }

    /// Get number of failed requests
    pub fn failed_requests(&self) -> u64 {
        self.failed_requests
    }

    /// Get total duration
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Get requests per second
    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }

    /// Get minimum response time
    pub fn min_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.min())
        }
    }

    /// Get maximum response time
    pub fn max_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.max())
        }
    }

    /// Get mean response time
    pub fn mean_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.mean() as u64)
        }
    }

    /// Get median response time
    pub fn median_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.value_at_percentile(50.0))
        }
    }

    /// Get 95th percentile response time
    pub fn p95_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.value_at_percentile(95.0))
        }
    }

    /// Get 99th percentile response time
    pub fn p99_response_time(&self) -> Duration {
        if self.response_times.len() == 0 {
            Duration::from_secs(0)
        } else {
            Duration::from_micros(self.response_times.value_at_percentile(99.0))
        }
    }

    /// Get error statistics
    pub fn error_stats(&self) -> Option<(HashMap<String, u64>, Vec<String>)> {
        if self.error_counts.is_empty() {
            None
        } else {
            Some((self.error_counts.clone(), self.error_messages.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_request_metrics_lifecycle() {
        let metrics = RequestMetrics::new();
        assert!(metrics.duration().is_none());
        assert!(metrics.status().is_none());
        assert!(metrics.error().is_none());

        let metrics = metrics.complete(StatusCode::OK);
        assert!(metrics.duration().is_some());
        assert_eq!(metrics.status(), Some(StatusCode::OK));
        assert!(metrics.error().is_none());

        // Test with explicit duration
        let mut metrics = RequestMetrics::new();
        let explicit_duration = Duration::from_millis(100);
        metrics.duration = Some(explicit_duration);
        let metrics = metrics.complete(StatusCode::OK);
        assert!(metrics.duration().unwrap() >= explicit_duration);
        assert_eq!(metrics.status(), Some(StatusCode::OK));
        assert!(metrics.error().is_none());
    }

    #[test]
    fn test_metrics_basic_stats() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record successful requests
        for _ in 0..5 {
            metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));
        }

        // Record failed requests
        for _ in 0..3 {
            metrics
                .record_request(RequestMetrics::new().complete(StatusCode::INTERNAL_SERVER_ERROR));
        }

        assert_eq!(metrics.total_requests(), 8);
        assert_eq!(metrics.successful_requests(), 5);
        assert_eq!(metrics.failed_requests(), 3);
    }

    #[test]
    fn test_metrics_response_times() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record requests with different response times
        let mut request = RequestMetrics::new();
        request.duration = Some(Duration::from_millis(100));
        metrics.record_request(request.complete(StatusCode::OK));

        let mut request = RequestMetrics::new();
        request.duration = Some(Duration::from_millis(200));
        metrics.record_request(request.complete(StatusCode::OK));

        let mut request = RequestMetrics::new();
        request.duration = Some(Duration::from_millis(300));
        metrics.record_request(request.complete(StatusCode::OK));

        // Check that response times are recorded correctly
        assert!(metrics.min_response_time() <= metrics.max_response_time());
        assert!(metrics.mean_response_time() >= metrics.min_response_time());
        assert!(metrics.mean_response_time() <= metrics.max_response_time());
        assert!(metrics.p95_response_time() >= metrics.median_response_time());
        assert!(metrics.p99_response_time() >= metrics.p95_response_time());
    }

    #[test]
    fn test_metrics_with_invalid_histogram() {
        let result = Histogram::<u64>::new(6); // 6 significant digits is invalid
        assert!(result.is_err());
    }

    #[test]
    fn test_metrics_error_handling() {
        let result = TestMetrics::new();
        assert!(result.is_ok());

        let mut metrics = TestMetrics::new().unwrap();
        let request_metrics = RequestMetrics::new();
        metrics.record_request(request_metrics);
        assert_eq!(metrics.total_requests(), 1);
    }

    #[test]
    fn test_metrics_with_zero_requests() {
        let metrics = TestMetrics::new().unwrap();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.successful_requests(), 0);
        assert_eq!(metrics.failed_requests(), 0);
        assert_eq!(metrics.requests_per_second(), 0.0);
    }

    #[test]
    fn test_metrics_no_error_stats() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record only successful requests
        metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));
        metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));

        assert_eq!(metrics.error_stats(), None);
        assert_eq!(metrics.total_requests(), 2);
        assert_eq!(metrics.successful_requests(), 2);
        assert_eq!(metrics.failed_requests(), 0);
    }

    #[test]
    fn test_metrics_unknown_errors() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record a request with no status and no error
        let request = RequestMetrics::new();
        metrics.record_request(request);

        let (error_counts, error_messages) = metrics.error_stats().unwrap();

        assert_eq!(metrics.failed_requests(), 1);
        assert_eq!(error_counts.get("Unknown Error").unwrap(), &1u64);
        assert_eq!(error_messages.len(), 1);
        assert_eq!(error_messages[0], "Unknown Error");
    }

    #[test]
    fn test_metrics_sub_millisecond_response_times() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record requests with sub-millisecond response times
        let mut request = RequestMetrics::new();
        thread::sleep(Duration::from_micros(100));
        request.duration = Some(request.start_time.elapsed());
        println!("Recording duration: {:?}", request.duration.unwrap());
        metrics.record_request(request.complete(StatusCode::OK));

        let mut request = RequestMetrics::new();
        thread::sleep(Duration::from_micros(300));
        request.duration = Some(request.start_time.elapsed());
        println!("Recording duration: {:?}", request.duration.unwrap());
        metrics.record_request(request.complete(StatusCode::OK));

        let mut request = RequestMetrics::new();
        thread::sleep(Duration::from_micros(500));
        request.duration = Some(request.start_time.elapsed());
        println!("Recording duration: {:?}", request.duration.unwrap());
        metrics.record_request(request.complete(StatusCode::OK));

        println!("Min value in histogram: {}", metrics.response_times.min());
        println!("Max value in histogram: {}", metrics.response_times.max());
        println!("Mean value in histogram: {}", metrics.response_times.mean());

        // Check that sub-millisecond response times are recorded correctly
        let min_time = metrics.min_response_time();
        let max_time = metrics.max_response_time();
        let median_time = metrics.median_response_time();
        let mean_time = metrics.mean_response_time();
        let p95_time = metrics.p95_response_time();
        let p99_time = metrics.p99_response_time();

        println!("Min time: {:?}", min_time);
        println!("Max time: {:?}", max_time);
        println!("Median time: {:?}", median_time);
        println!("Mean time: {:?}", mean_time);
        println!("P95 time: {:?}", p95_time);
        println!("P99 time: {:?}", p99_time);

        assert!(
            min_time >= Duration::from_micros(100),
            "Min time {:?} should be >= 100µs",
            min_time
        );
        assert!(
            max_time >= Duration::from_micros(500),
            "Max time {:?} should be >= 500µs",
            max_time
        );
        assert!(
            median_time >= Duration::from_micros(300),
            "Median time {:?} should be >= 300µs",
            median_time
        );
        assert!(
            mean_time >= Duration::from_micros(300),
            "Mean time {:?} should be >= 300µs",
            mean_time
        );
        assert!(
            p95_time >= Duration::from_micros(500),
            "P95 time {:?} should be >= 500µs",
            p95_time
        );
        assert!(
            p99_time >= Duration::from_micros(500),
            "P99 time {:?} should be >= 500µs",
            p99_time
        );
    }

    #[tokio::test]
    async fn test_metrics_requests_per_second() {
        let mut metrics = TestMetrics::new().unwrap();

        // Record 20 requests
        for _ in 0..20 {
            metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));
        }

        // Test with a very short duration (sub-second)
        let short_duration = Duration::from_secs_f64(0.006573375);
        metrics.finalize(short_duration).await.unwrap();
        assert!(
            (metrics.requests_per_second() - 3042.86).abs() < 1.0,
            "Expected RPS ~3042.86, got {}",
            metrics.requests_per_second()
        );

        // Reset metrics
        let mut metrics = TestMetrics::new().unwrap();
        for _ in 0..50 {
            metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));
        }

        // Test with a longer duration
        let long_duration = Duration::from_secs_f64(0.075828625);
        metrics.finalize(long_duration).await.unwrap();
        assert!(
            (metrics.requests_per_second() - 659.38).abs() < 1.0,
            "Expected RPS ~659.38, got {}",
            metrics.requests_per_second()
        );

        // Test with zero duration
        let mut metrics = TestMetrics::new().unwrap();
        metrics.record_request(RequestMetrics::new().complete(StatusCode::OK));
        metrics.finalize(Duration::from_secs(0)).await.unwrap();
        assert_eq!(metrics.requests_per_second(), 0.0);
    }
}
