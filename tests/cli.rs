use assert_cmd::Command;

#[test]
fn missing_flag_values_identify_the_flag_and_preserve_encoding() {
    for flag in ["--workspace", "--path", "--config", "--format"] {
        let mut args = vec!["config", "explain"];
        if flag != "--workspace" {
            args.extend(["--workspace", "/does-not-exist-AS-M002"]);
        }
        if flag != "--path" {
            args.extend(["--path", "future.ts"]);
        }
        if flag != "--format" {
            args.extend(["--format", "json"]);
        }
        args.push(flag);
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_anti-slop"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let value = if flag == "--format" {
            decode_toon(&output)
        } else {
            json(&output)
        };
        let message = value["error"]["message"].as_str().unwrap();
        assert!(message.contains(flag), "{flag}: {message}");
        if flag != "--format" {
            assert!(!message.contains("--format"), "{flag}: {message}");
        }
        assert_eq!(value["error"]["code"], "invalid_input");
    }
}

fn explain(root: &std::path::Path, path: &str, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .args(["config", "explain", "--workspace"])
        .arg(root)
        .args(["--path", path])
        .args(args)
        .output()
        .unwrap()
}
fn json(output: &std::process::Output) -> serde_json::Value {
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn decode_toon(output: &std::process::Output) -> serde_json::Value {
    use std::io::Write;
    let mut child = std::process::Command::new("node")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tools/config-conformance/decode.mjs"
        ))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&output.stdout)
        .unwrap();
    let decoded = child.wait_with_output().unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    serde_json::from_slice(&decoded.stdout).unwrap()
}

#[test]
fn default_toon_and_explicit_json_have_identical_real_cli_views() {
    let root = tempfile::TempDir::new().unwrap();
    let toon = explain(root.path(), "future/é😀,:[]\"\\.ts", &[]);
    let explicit = explain(root.path(), "future/é😀,:[]\"\\.ts", &["--format", "toon"]);
    let as_json = explain(root.path(), "future/é😀,:[]\"\\.ts", &["--format", "json"]);
    assert!(toon.status.success());
    assert_eq!(toon.stdout, explicit.stdout);
    assert_eq!(decode_toon(&toon), json(&as_json));
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn real_cli_explicit_selection_ordered_overrides_and_cwd_are_read_only() {
    let root = tempfile::TempDir::new().unwrap();
    let other = tempfile::TempDir::new().unwrap();
    let config = serde_json::json!({"$schema":anti_slop::config::SCHEMA_URL,"rules":{"R1":"off","R4":"block"},"overrides":[
        {"include":["src/**"],"exclude":["src/generated/**"],"rules":{"R1":"advisory","R2":"off"}},
        {"include":["src/**"],"rules":{"R1":"block","R3":"off"}}
    ]}).to_string();
    std::fs::write(root.path().join("policy.jsonc"), &config).unwrap();
    std::fs::write(
        root.path().join(".anti-slop.jsonc"),
        "invalid root ignored by explicit selection",
    )
    .unwrap();
    for (path, matches, expected) in [
        ("src/file.ts", vec![0, 1], "block"),
        ("src/generated/file.ts", vec![1], "block"),
        ("other.ts", vec![], "off"),
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_anti-slop"))
            .current_dir(other.path())
            .args(["config", "explain", "--workspace"])
            .arg(root.path())
            .args([
                "--path",
                path,
                "--config",
                "policy.jsonc",
                "--format",
                "json",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let view = json(&output);
        assert_eq!(
            view["config"]["matched_overrides"],
            serde_json::json!(matches)
        );
        assert_eq!(view["config"]["rules"][0]["policy"], expected);
        assert_eq!(view["config"]["rules"][3]["policy"], "block");
        assert_eq!(
            view["config"]["rules"][0]["origin"]["kind"],
            if path == "other.ts" {
                "root"
            } else {
                "override"
            }
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("policy.jsonc")).unwrap(),
            config
        );
    }
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 2);
    assert_eq!(std::fs::read_dir(other.path()).unwrap().count(), 0);
}

#[test]
fn inspection_argument_errors_are_structured_redacted_and_precede_file_access() {
    let root = tempfile::TempDir::new().unwrap();
    for args in [
        vec!["--unknown-secret-value", "--format", "json"],
        vec!["--format", "xml"],
        vec!["--format=JSON"],
        vec!["--format", "json", "--format", "toon"],
    ] {
        let output = explain(&root.path().join("nonexistent"), "future.ts", &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stderr.is_empty());
        let value = if args[0] == "--unknown-secret-value" {
            json(&output)
        } else {
            decode_toon(&output)
        };
        assert_eq!(value["error"]["code"], "invalid_input");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("secret-value"));
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .args(["config", "explain", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["error"]["code"], "invalid_input");
}

#[test]
fn invalid_existing_configuration_and_explicit_missing_file_have_distinct_exits() {
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(
        root.path().join(".anti-slop.jsonc"),
        format!(
            "{{\"$schema\":\"{}\",\n\"provider\":\"secret-test-value\"}}",
            anti_slop::config::SCHEMA_URL
        ),
    )
    .unwrap();
    let invalid = explain(root.path(), "future.ts", &["--format", "json"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(json(&invalid)["error"]["location"]["line"], 2);
    assert!(!String::from_utf8_lossy(&invalid.stdout).contains("secret-test-value"));
    let missing = explain(root.path(), "future.ts", &["--config", "missing.jsonc"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(decode_toon(&missing)["error"]["code"], "io");
    let path = explain(root.path(), "../escape.ts", &["--format", "json"]);
    assert_eq!(path.status.code(), Some(2));
    assert_eq!(json(&path)["error"]["code"], "invalid_input");
}

#[test]
fn config_help_describes_the_exact_local_inspection_surface() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .args(["config", "explain", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "--workspace",
        "--path",
        "--config",
        "--format",
        "toon",
        "json",
        "Examples:",
    ] {
        assert!(text.contains(expected));
    }
}

#[test]
fn config_explain_returns_defaults_as_json() {
    let root = tempfile::TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_anti-slop"))
        .args(["config", "explain", "--workspace"])
        .arg(root.path())
        .args(["--path", "future.ts", "--format", "json"])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    let view: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(view["config"]["source"]["kind"], "built_in");
    assert_eq!(view["config"]["rules"].as_array().unwrap().len(), 9);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

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
