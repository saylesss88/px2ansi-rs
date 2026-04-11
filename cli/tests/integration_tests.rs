use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    Command::cargo_bin("px2ansi-rs").unwrap()
}

#[test]
fn help_exits_successfully() {
    cmd().arg("--help").assert().success();
}

#[test]
fn convert_help_shows_style_flag() {
    cmd()
        .args(["convert", "--help"])
        .assert()
        .success()
        .stdout(contains("--style"));
}

#[test]
fn show_help_exits_successfully() {
    cmd().args(["show", "--help"]).assert().success();
}

#[test]
fn invalid_style_shows_error() {
    cmd()
        .args(["convert", "input.png", "--style", "invalid"])
        .assert()
        .failure()
        .stderr(contains("invalid"));
}

#[test]
fn convert_nonexistent_file_fails_gracefully() {
    cmd()
        .args(["convert", "nonexistent.png"])
        .assert()
        .failure();
}

#[test]
fn list_with_missing_index_fails_gracefully() {
    cmd()
        .args(["list", "-I", "nonexistent_index.json"])
        .assert()
        .failure();
}

#[test]
fn version_flag_exits_successfully() {
    cmd().arg("--version").assert().success();
}
