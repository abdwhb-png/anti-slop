use anti_slop::{
    config::{self, ConfigError, ErrorCode, Location},
    output::{self, Format, Inspection},
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[test]
fn pinned_specification_examples_qualify_the_selected_codec_forms() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/output/reference-v4.1.json")).unwrap();
    for case in cases.as_array().unwrap() {
        assert_eq!(
            serde_toon::to_string(&case["input"]).unwrap(),
            case["toon"].as_str().unwrap()
        );
        assert_eq!(
            independent_decode(case["toon"].as_str().unwrap()),
            case["input"]
        );
    }
}

fn independent_decode(text: &str) -> serde_json::Value {
    let mut child = Command::new("node")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tools/config-conformance/decode.mjs"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "reference decode failed: {}\n{text}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}
fn check(view: &Inspection<'_>) {
    let toon = output::encode(view, Format::Toon).unwrap();
    assert_eq!(toon, output::encode(view, Format::Toon).unwrap());
    assert!(!toon.ends_with('\n'));
    assert!(
        toon.lines()
            .all(|line| !line.ends_with(' ') && !line.starts_with('\t'))
    );
    let json = output::encode(view, Format::Json).unwrap();
    assert_eq!(
        independent_decode(&toon),
        serde_json::from_str::<serde_json::Value>(&json).unwrap()
    );
}

#[test]
fn default_and_mixed_origins_interoperate_with_independent_reference_decoder() {
    let root = tempfile::TempDir::new().unwrap();
    let defaults = config::resolve(root.path(), Path::new("future.ts"), None).unwrap();
    check(&Inspection::Config {
        config: defaults.view(),
    });
    std::fs::write(root.path().join(".anti-slop.jsonc"), serde_json::json!({"$schema":config::SCHEMA_URL,"rules":{"R1":"off"},"overrides":[{"include":["**"],"rules":{"R2":"block","R3":"off"}}]}).to_string()).unwrap();
    let mixed = config::resolve(root.path(), Path::new("src/é😀.ts"), None).unwrap();
    check(&Inspection::Config {
        config: mixed.view(),
    });
    assert_eq!(mixed.view().matched_overrides, [0]);
}

#[test]
fn all_inspection_string_forms_and_located_null_errors_interoperate() {
    let root = tempfile::TempDir::new().unwrap();
    let controls: String = (1..32).map(char::from).collect();
    for path in [
        "true",
        "null",
        "05",
        "-x",
        "#x",
        "+1",
        " spaced ",
        "punct,:[]{}\"\\é😀",
        "é\u{a0}",
        &controls,
    ] {
        let snapshot = config::resolve(root.path(), Path::new(path), None).unwrap();
        check(&Inspection::Config {
            config: snapshot.view(),
        });
    }
    for location in [
        None,
        Some(Location {
            file: "comma,:\"\\\né😀".into(),
            line: Some(1048576),
            column: None,
        }),
    ] {
        let error = ConfigError {
            code: ErrorCode::InvalidConfiguration,
            message: "configuration requires correction".into(),
            location,
        };
        check(&Inspection::Error {
            error: &error,
            help: "Correct the indicated entry.",
        });
    }
}
