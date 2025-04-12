//! Lode Core - High Performance Load Testing Library
//!
//! This library provides the core functionality for executing load tests against HTTP APIs.
//! It is designed to be efficient, reliable, and easy to integrate into both CLI and REST API applications.

pub mod config; // Load test configuration
pub mod engine; // Test execution engine
pub mod error; // Error types and handling
pub mod http; // HTTP client and request handling
pub mod metrics; // Performance metrics collection and analysis
pub mod report; // Test results and reporting
pub mod telemetry; // Structured logging and telemetry

pub use config::LoadTestConfig;
pub use engine::LoadTestEngine;
pub use error::Error;
pub use metrics::TestMetrics;
pub use telemetry::{
    get_stderr_subscriber, get_stdout_subscriber, get_subscriber, init_subscriber,
};

pub type Result<T> = std::result::Result<T, Error>;
