use crate::helpers::{bin_cmd, temp_home_with_config, ASCII_CONFIG};

#[test]
fn print_default_config() {
    let home = temp_home_with_config(None);
    bin_cmd(&home)
        .arg("print")
        .assert()
        .success()
        .stdout(predicates::str::contains("mode = \"auto\""));
}

#[test]
fn print_custom_config() {
    let home = temp_home_with_config(Some(ASCII_CONFIG));
    bin_cmd(&home)
        .arg("print")
        .assert()
        .success()
        .stdout(predicates::str::contains("mode = \"ascii\""));
}
