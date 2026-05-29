use predicates::prelude::PredicateBooleanExt;

use crate::helpers::{bin_cmd, temp_home_with_config, ASCII_CONFIG};

#[test]
fn check_valid_config() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    bin_cmd(&home)
        .arg("check")
        .assert()
        .success()
        .stdout(predicates::str::contains("valid"));
}

#[test]
fn check_missing_config() {
    let home = temp_home_with_config(None);
    bin_cmd(&home)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("not found"));
}

#[test]
fn check_empty_segments() {
    let config = r#"
[style]
mode = "ascii"

[[segments]]
"#;
    let home = temp_home_with_config(Some(config));
    bin_cmd(&home)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid").or(predicates::str::contains("Invalid")));
}

#[test]
fn check_duplicate_segment_id() {
    let config = r#"
[style]
mode = "ascii"

[[segments]]
id = "token_usage"
enabled = true
[segments.icon]
emoji = "🪙"
ascii = "$"
[segments.options]

[[segments]]
id = "token_usage"
enabled = true
[segments.icon]
emoji = "🪙"
ascii = "$"
[segments.options]
"#;
    let home = temp_home_with_config(Some(config));
    bin_cmd(&home)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Duplicate"));
}

#[test]
fn check_invalid_segment_id() {
    let config = r#"
[style]
mode = "ascii"

[[segments]]
id = "nonexistent_segment"
enabled = true
[segments.icon]
emoji = "❌"
ascii = "!"
[segments.options]
"#;
    let home = temp_home_with_config(Some(config));
    bin_cmd(&home)
        .arg("check")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid segment ID"));
}
