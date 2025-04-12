use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_basic_load_test() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    Command::cargo_bin("lode-cli")?
        .arg("--url")
        .arg(url)
        .arg("--requests")
        .arg("10")
        .arg("--concurrency")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Requests: 10"))
        .stdout(predicate::str::contains("Successful Requests: 10"));

    Ok(())
}

#[tokio::test]
async fn test_json_output() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    Command::cargo_bin("lode-cli")?
        .arg("--url")
        .arg(url)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_requests\":"))
        .stdout(predicate::str::contains("\"successful_requests\":"));

    Ok(())
}

#[tokio::test]
async fn test_post_with_body() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    Command::cargo_bin("lode-cli")?
        .arg("--url")
        .arg(url)
        .arg("--method")
        .arg("POST")
        .arg("--body")
        .arg(r#"{"test": "data"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Requests:"));

    Ok(())
}

#[tokio::test]
async fn test_with_headers() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let url = format!("{}/test", mock_server.uri());

    Command::cargo_bin("lode-cli")?
        .arg("--url")
        .arg(url)
        .arg("-H")
        .arg("Authorization:Bearer token")
        .arg("-H")
        .arg("Content-Type:application/json")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Requests:"));

    Ok(())
}
