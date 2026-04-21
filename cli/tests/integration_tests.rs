use assert_cmd::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Helper to create a command, bubbling up the error if the binary isn't found
fn cmd() -> Result<Command, Box<dyn std::error::Error>> {
    let command = Command::cargo_bin("px2ansi-rs")?;
    Ok(command)
}

#[test]
fn help_exits_successfully() -> TestResult {
    cmd()?.arg("--help").assert().success();
    Ok(())
}

#[test]
fn convert_help_shows_style_flag() -> TestResult {
    cmd()?
        .args(["convert", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("--style"));
    Ok(())
}

#[test]
fn invalid_style_shows_error() -> TestResult {
    cmd()?
        .args(["convert", "input.png", "--style", "invalid"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid"));
    Ok(())
}

#[test]
fn version_flag_exits_successfully() -> TestResult {
    cmd()?.arg("--version").assert().success();
    Ok(())
}
