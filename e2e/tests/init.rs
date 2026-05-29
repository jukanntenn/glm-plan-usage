use crate::helpers::{bin_cmd, temp_home_with_config};

#[test]
fn init_creates_config_file() {
    let home = temp_home_with_config(None);
    bin_cmd(&home)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("Created config"));

    let config_path = home
        .path()
        .join(".claude")
        .join("glm-plan-usage")
        .join("config.toml");
    assert!(config_path.exists(), "config file should be created");
}

#[test]
fn init_already_exists() {
    let home = temp_home_with_config(Some("[style]\nmode = \"emoji\"\n"));
    bin_cmd(&home)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("already exists"));
}
