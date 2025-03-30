use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lode_core::{
    config::LoadTestConfig,
    engine::LoadTestEngine,
    http::DefaultHttpClient,
    report::Report,
    telemetry::{get_stdout_subscriber, init_subscriber},
};
use std::time::Duration;

use lode_cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize telemetry only if --no-capture is used
    if cli.no_capture {
        let subscriber = get_stdout_subscriber("lode-cli".into(), "info".into());
        init_subscriber(subscriber);
    }

    // Create progress bar
    let pb = ProgressBar::new(cli.requests as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );

    // Create config
    let mut config = LoadTestConfig::new(
        cli.url,
        cli.method.parse()?,
        cli.requests as usize,
        cli.concurrency as usize,
        Duration::from_secs(cli.timeout),
    )?;

    // Set body if provided
    if let Some(body) = cli.body {
        config.body = Some(body);
    }

    // Parse headers if provided
    if let Some(headers) = cli.headers {
        config.headers = headers
            .iter()
            .map(|h| {
                let parts: Vec<&str> = h.split(':').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid header format: {}", h);
                }
                Ok((parts[0].to_string(), parts[1].to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
    }

    // Create engine and run test
    let client = DefaultHttpClient::new()?;
    let engine = LoadTestEngine::new(client)?;
    let result = engine
        .run(
            config.method.into(),
            config.url,
            config.requests as u64,
            config.concurrency as u64,
            config.timeout,
            config.headers,
            config.body,
            Some(pb),
        )
        .await?;

    // Create report
    let report = Report::from_metrics(result).await?;

    // Print results based on format
    match cli.format.to_lowercase().as_str() {
        "json" => println!("{}", report.as_json()?),
        _ => println!("{}", report.as_string()),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::Parser;

    #[test]
    fn test_cli_parsing() -> Result<()> {
        let args = vec![
            "lode",
            "--url",
            "https://example.com",
            "--requests",
            "100",
            "--concurrency",
            "10",
            "--method",
            "GET",
            "--timeout",
            "30",
        ];

        let cli = crate::Cli::try_parse_from(args)?;
        assert_eq!(cli.url, "https://example.com");
        assert_eq!(cli.requests, 100);
        assert_eq!(cli.concurrency, 10);
        assert_eq!(cli.method, "GET");
        assert_eq!(cli.timeout, 30);
        assert_eq!(cli.format, "text");
        Ok(())
    }

    #[test]
    fn test_cli_with_headers() -> Result<()> {
        let args = vec![
            "lode",
            "--url",
            "https://example.com",
            "--headers",
            "Authorization:Bearer token,Content-Type:application/json",
        ];

        let cli = crate::Cli::try_parse_from(args)?;
        let headers = cli.headers.unwrap();
        assert_eq!(headers.len(), 2);
        assert!(headers.contains(&"Authorization:Bearer token".to_string()));
        assert!(headers.contains(&"Content-Type:application/json".to_string()));
        Ok(())
    }

    #[test]
    fn test_cli_with_body() -> Result<()> {
        let args = vec![
            "lode",
            "--url",
            "https://example.com",
            "--body",
            r#"{"key": "value"}"#,
        ];

        let cli = crate::Cli::try_parse_from(args)?;
        assert_eq!(cli.body.unwrap(), r#"{"key": "value"}"#);
        Ok(())
    }

    #[test]
    fn test_cli_output_formats() -> Result<()> {
        let args = vec!["lode", "--url", "https://example.com", "--format", "json"];

        let cli = crate::Cli::try_parse_from(args)?;
        assert_eq!(cli.format, "json");
        Ok(())
    }
}
