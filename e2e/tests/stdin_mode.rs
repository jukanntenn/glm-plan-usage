use crate::helpers::{bin_cmd, read_fixture, temp_home_with_config, ASCII_CONFIG};
use httpmock::prelude::*;

fn mock_zhipu_url(server: &MockServer) -> String {
    format!("http://127.0.0.1:{}/zhipu/api/anthropic", server.port())
}

fn mock_api_path() -> &'static str {
    "/zhipu/api/monitor/usage/quota/limit"
}

#[test]
fn stdin_with_mocked_api() {
    let server = MockServer::start();
    let api_response = read_fixture("api_response.json");

    server.mock(|when, then| {
        when.path(mock_api_path())
            .header("Authorization", "Bearer test-token");
        then.status(200).body(&api_response);
    });

    let home = temp_home_with_config(Some(ASCII_CONFIG));
    let stdin_input = read_fixture("stdin_minimal.json");

    bin_cmd(&home)
        .arg("--no-cache")
        .env("ANTHROPIC_AUTH_TOKEN", "test-token")
        .env("ANTHROPIC_BASE_URL", mock_zhipu_url(&server))
        .write_stdin(stdin_input)
        .assert()
        .success()
        .stdout(predicates::str::contains("$ 50%"))
        .stdout(predicates::str::contains("# 30/100"));
}

#[test]
fn stdin_full_input_with_mocked_api() {
    let server = MockServer::start();
    let api_response = read_fixture("api_response.json");

    server.mock(|when, then| {
        when.path(mock_api_path());
        then.status(200).body(&api_response);
    });

    let home = temp_home_with_config(Some(ASCII_CONFIG));
    let stdin_input = read_fixture("stdin_full.json");

    bin_cmd(&home)
        .arg("--no-cache")
        .env("ANTHROPIC_AUTH_TOKEN", "test-token")
        .env("ANTHROPIC_BASE_URL", mock_zhipu_url(&server))
        .write_stdin(stdin_input)
        .assert()
        .success()
        .stdout(predicates::str::contains("50%"));
}

#[test]
fn stdin_api_returns_empty_limits() {
    let server = MockServer::start();
    let empty_response = r#"{"code":200,"msg":"ok","success":true,"data":{"limits":[]}}"#;

    server.mock(|when, then| {
        when.path(mock_api_path());
        then.status(200).body(empty_response);
    });

    let home = temp_home_with_config(Some(ASCII_CONFIG));
    let stdin_input = read_fixture("stdin_minimal.json");

    bin_cmd(&home)
        .arg("--no-cache")
        .env("ANTHROPIC_AUTH_TOKEN", "test-token")
        .env("ANTHROPIC_BASE_URL", mock_zhipu_url(&server))
        .write_stdin(stdin_input)
        .assert()
        .success();
}
