use crate::helpers::{bin_cmd, temp_home_with_config, ASCII_CONFIG};

#[test]
fn empty_stdin_no_panic() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    bin_cmd(&home)
        .arg("--no-cache")
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn invalid_json_stdin_no_panic() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    bin_cmd(&home)
        .arg("--no-cache")
        .write_stdin("this is not json at all!!!")
        .assert()
        .success();
}

#[test]
fn no_api_env_vars_graceful_degradation() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    let stdin = r#"{"model":{"id":"test"}}"#;

    bin_cmd(&home)
        .arg("--no-cache")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("ANTHROPIC_BASE_URL")
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}

#[test]
fn api_unreachable_graceful_degradation() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    let stdin = r#"{"model":{"id":"test"}}"#;

    bin_cmd(&home)
        .arg("--no-cache")
        .env("ANTHROPIC_AUTH_TOKEN", "test-token")
        .env(
            "ANTHROPIC_BASE_URL",
            "https://open.bigmodel.cn/api/anthropic",
        )
        .write_stdin(stdin)
        .assert()
        .success();
}

#[test]
fn verbose_mode_stderr_on_invalid_json() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    bin_cmd(&home)
        .arg("--verbose")
        .arg("--no-cache")
        .write_stdin("not json")
        .assert()
        .success()
        .stderr(predicates::str::contains("Error parsing input JSON"));
}

#[test]
fn missing_config_still_works_with_stdin() {
    let home = temp_home_with_config(None);
    let stdin = r#"{"model":{"id":"test"}}"#;

    bin_cmd(&home)
        .arg("--no-cache")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .write_stdin(stdin)
        .assert()
        .success();
}
