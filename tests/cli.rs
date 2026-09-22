use assert_cmd::Command;

#[test]
fn unavailable_analysis_is_rejected() {
    Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .arg("analyze")
        .assert()
        .code(2)
        .stdout("");
}

#[test]
fn invocation_without_arguments_is_not_a_successful_analysis() {
    Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .assert()
        .code(2)
        .stdout("");
}

#[test]
fn help_is_available() {
    let result = Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .arg("--help")
        .assert()
        .success()
        .stderr("");

    assert!(!result.get_output().stdout.is_empty());
}

#[test]
fn version_matches_package_metadata() {
    Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .arg("--version")
        .assert()
        .success()
        .stdout(concat!(
            env!("CARGO_PKG_NAME"),
            " ",
            env!("CARGO_PKG_VERSION"),
            "\n"
        ))
        .stderr("");
}
