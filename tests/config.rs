//! Qualification of configuration dependencies against the mission's input contract.

use anti_slop::config::{ErrorCode, MAX_FILE_BYTES, Origin, Policy, RuleId, Source, resolve};
use std::{fs, path::Path};
use tempfile::TempDir;

const SCHEMA: &str =
    "https://github.com/abdwhb-png/anti-slop/releases/download/config-v1.0.0/anti-slop.schema.json";

fn install(root: &Path, content: &str) {
    fs::write(root.join(".anti-slop.jsonc"), content).unwrap();
}

fn override_document(pattern: &str) -> String {
    serde_json::json!({"$schema": SCHEMA, "overrides": [{"include": [pattern], "rules": {"R1": "off"}}]}).to_string()
}

#[test]
fn absent_root_uses_advisory_defaults_without_writes() {
    let root = TempDir::new().unwrap();
    let snapshot = resolve(root.path(), Path::new("future/file.ts"), None).unwrap();
    assert_eq!(snapshot.view().rules.len(), 9);
    assert!(
        snapshot
            .view()
            .rules
            .iter()
            .all(|s| s.policy == Policy::Advisory && matches!(s.origin, Origin::BuiltIn { .. }))
    );
    assert_eq!(snapshot.view().source, Source::BuiltIn);
    assert!(snapshot.captured_bytes().is_none());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn per_path_disagreement_keeps_both_views_and_transmission_controls_are_rejected() {
    let root = TempDir::new().unwrap();
    install(root.path(), &serde_json::json!({"$schema":SCHEMA,"rules":{"R1":"block"},"overrides":[{"include":["**/*.ts"],"rules":{"R1":"off"}}]}).to_string());
    let ts = resolve(root.path(), Path::new("src/future.ts"), None).unwrap();
    let php = resolve(root.path(), Path::new("src/future.php"), None).unwrap();
    assert_eq!(ts.view().rules[0].policy, Policy::Off);
    assert!(matches!(
        ts.view().rules[0].origin,
        Origin::Override { index: 0, .. }
    ));
    assert_eq!(php.view().rules[0].policy, Policy::Block);
    assert!(matches!(php.view().rules[0].origin, Origin::Root { .. }));
    assert_eq!(ts.view().source, php.view().source);
    for field in ["authorization", "transmission", "scope", "qualified"] {
        let document =
            serde_json::json!({"$schema":SCHEMA,field:"controlled-secret-authorization"});
        assert!(document.get(field).is_some());
        install(root.path(), &document.to_string());
        let error = resolve(root.path(), Path::new("src/future.ts"), None).unwrap_err();
        assert_eq!(error.exit_code(), 2);
        assert!(
            !serde_json::to_string(&error)
                .unwrap()
                .contains("controlled-secret-authorization")
        );
    }
    // Later rejected loads cannot mutate either captured per-path view.
    assert_eq!(ts.view().rules[0].policy, Policy::Off);
    assert_eq!(php.view().rules[0].policy, Policy::Block);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn question_mark_matches_one_unicode_scalar() {
    let root = TempDir::new().unwrap();
    install(root.path(), &override_document("?.ts"));
    for (path, expected) in [
        ("é.ts", Policy::Off),
        ("😀.ts", Policy::Off),
        ("ab.ts", Policy::Advisory),
        ("é.ts", Policy::Advisory),
    ] {
        let snapshot = resolve(root.path(), Path::new(path), None).unwrap();
        assert_eq!(snapshot.view().rules[0].policy, expected, "{path}");
    }
}

#[test]
fn backslash_is_a_literal_filename_character() {
    let root = TempDir::new().unwrap();
    install(root.path(), &override_document(r"a\b.ts"));
    let snapshot = resolve(root.path(), Path::new(r"a\b.ts"), None).unwrap();
    assert_eq!(snapshot.view().rules[0].policy, Policy::Off);
    assert_eq!(
        resolve(root.path(), Path::new("a/b.ts"), None)
            .unwrap()
            .view()
            .rules[0]
            .policy,
        Policy::Advisory
    );
}

#[test]
fn jsonc_comments_and_trailing_commas_are_accepted() {
    let root = TempDir::new().unwrap();
    install(
        root.path(),
        &format!(
            r#"{{
        // local policy
        "$schema": "{SCHEMA}",
        "rules": {{ "R1": "off", }},
    }}"#
        ),
    );
    let result = resolve(root.path(), Path::new("src/file.ts"), None);
    assert!(result.is_ok(), "valid JSONC failed: {result:?}");
    assert_eq!(result.unwrap().view().rules[0].rule, RuleId::R1);
}

#[test]
fn duplicate_decoded_keys_are_rejected_even_in_unmatched_overrides() {
    let root = TempDir::new().unwrap();
    for document in [
        format!(r#"{{"$schema":"{SCHEMA}","rules":{{"R1":"off","R\u0031":"block"}}}}"#),
        format!(
            r#"{{"$schema":"{SCHEMA}","overrides":[{{"include":["other/**"],"rules":{{"R1":"off","R1":"block"}}}}]}}"#
        ),
        format!(r#"{{"$schema":"{SCHEMA}","$schema":"{SCHEMA}"}}"#),
    ] {
        install(root.path(), &document);
        let result = resolve(root.path(), Path::new("src/file.ts"), None);
        assert!(result.is_err(), "duplicate key was accepted: {result:?}");
        let error = result.unwrap_err();
        assert_eq!(error.exit_code(), 2);
        assert!(error.location.unwrap().column.unwrap() > 1);
    }
}

#[test]
fn ordered_overrides_replace_policies_and_preserve_unrelated_rules_and_origins() {
    let root = TempDir::new().unwrap();
    install(
        root.path(),
        &serde_json::json!({
            "$schema": SCHEMA,
            "rules": {"R1":"off","R2":"block","R4":"off"},
            "overrides": [
                {"include":["**/*.ts"],"rules":{"R1":"advisory","R2":"off"}},
                {"include":["src/**"],"exclude":["**/generated/**"],"rules":{"R1":"block"}},
                {"include":["src/generated/**"],"rules":{"R1":"off","R3":"block"}}
            ]
        })
        .to_string(),
    );
    let ordinary = resolve(root.path(), Path::new("src/file.ts"), None).unwrap();
    assert_eq!(ordinary.view().matched_overrides, [0, 1]);
    assert_eq!(ordinary.view().rules[0].policy, Policy::Block);
    assert!(matches!(
        ordinary.view().rules[0].origin,
        Origin::Override { index: 1, .. }
    ));
    assert_eq!(ordinary.view().rules[1].policy, Policy::Off);
    assert!(matches!(
        ordinary.view().rules[1].origin,
        Origin::Override { index: 0, .. }
    ));
    assert_eq!(ordinary.view().rules[2].policy, Policy::Advisory);
    assert!(matches!(
        ordinary.view().rules[2].origin,
        Origin::BuiltIn { .. }
    ));
    assert_eq!(ordinary.view().rules[3].policy, Policy::Off);
    assert!(matches!(
        ordinary.view().rules[3].origin,
        Origin::Root { .. }
    ));
    let generated = resolve(root.path(), Path::new("src/generated/file.ts"), None).unwrap();
    assert_eq!(generated.view().matched_overrides, [0, 2]);
    assert_eq!(generated.view().rules[0].policy, Policy::Off);
    assert_eq!(generated.view().rules[2].policy, Policy::Block);
    assert_eq!(
        resolve(root.path(), Path::new("README.md"), None)
            .unwrap()
            .view()
            .matched_overrides,
        Vec::<usize>::new()
    );
    assert_eq!(
        resolve(root.path(), Path::new(".hidden.ts"), None)
            .unwrap()
            .view()
            .matched_overrides,
        [0]
    );
}

#[test]
fn glob_dialect_handles_unicode_classes_braces_and_literal_control_characters() {
    let root = TempDir::new().unwrap();
    for (pattern, yes, no) in [
        ("[é-ô].{ts,tsx}", "é.tsx", "a.tsx"),
        ("[!a].ts", "é.ts", "a.ts"),
        ("src[!a]file.ts", "srcbfile.ts", "src/file.ts"),
        ("src/[[]id].ts", "src/[id].ts", "src/aid.ts"),
        ("[]].ts", "].ts", "x.ts"),
        ("a{,b}.ts", "a.ts", "ac.ts"),
        ("src/**", "src/nested/a.ts", "other/a.ts"),
        ("*.ts", "line\nname.ts", "nested/name.ts"),
        ("**/*.ts", "nested/line\nname.ts", "NAME.TS"),
        ("**/*.ts", "a.ts", "a.tsx"),
    ] {
        install(root.path(), &override_document(pattern));
        assert_eq!(
            resolve(root.path(), Path::new(yes), None)
                .unwrap()
                .view()
                .rules[0]
                .policy,
            Policy::Off,
            "{pattern} => {yes}"
        );
        assert_eq!(
            resolve(root.path(), Path::new(no), None)
                .unwrap()
                .view()
                .rules[0]
                .policy,
            Policy::Advisory,
            "{pattern} => {no}"
        );
    }
    for invalid in [
        "!src/**",
        "a**b",
        "***",
        "[abc",
        "{a,{b,c}}",
        "",
        "[a/]",
        "{a,b",
    ] {
        install(root.path(), &override_document(invalid));
        assert_eq!(
            resolve(root.path(), Path::new("a.ts"), None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidPattern,
            "{invalid}"
        );
    }
}

#[test]
fn recursive_segments_keep_their_meaning_inside_alternatives() {
    let root = TempDir::new().unwrap();
    for (pattern, paths) in [
        ("{**,a}", vec!["x", "n/x", "n/deep/x"]),
        ("src/{**,a}", vec!["src/x", "src/n/x", "src/n/deep/x"]),
        (
            "src/{**,a}/file",
            vec!["src/file", "src/n/file", "src/a/file"],
        ),
        ("{src/**,lib/**}/file", vec!["src/file", "lib/n/file"]),
    ] {
        install(root.path(), &override_document(pattern));
        for path in paths {
            assert_eq!(
                resolve(root.path(), Path::new(path), None)
                    .unwrap()
                    .view()
                    .rules[0]
                    .policy,
                Policy::Off,
                "{pattern} => {path}"
            );
        }
    }
}

#[test]
fn commas_and_partial_alternatives_do_not_make_recursive_segment_boundaries() {
    let root = TempDir::new().unwrap();
    for pattern in ["a,**", "**,a", "a{**,b}", "{**,a}x", "{**,}**"] {
        install(root.path(), &override_document(pattern));
        assert_eq!(
            resolve(root.path(), Path::new("a,x"), None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidPattern,
            "{pattern}"
        );
    }
}

#[test]
fn alternatives_compile_compositionally_at_the_pattern_limit() {
    let root = TempDir::new().unwrap();
    let pattern = "{a,b}".repeat(800);
    install(root.path(), &override_document(&pattern));
    assert_eq!(
        resolve(root.path(), Path::new(&"a".repeat(800)), None)
            .unwrap()
            .view()
            .rules[0]
            .policy,
        Policy::Off
    );
    assert_eq!(
        resolve(root.path(), Path::new(&"c".repeat(800)), None)
            .unwrap()
            .view()
            .rules[0]
            .policy,
        Policy::Advisory
    );
}

#[test]
fn invalid_jsonc_fields_values_and_contracts_fail_without_echoing_credentials() {
    let root = TempDir::new().unwrap();
    for invalid in [
        serde_json::json!({"$schema":SCHEMA,"provider":{"api_key":"secret-test-value"}})
            .to_string(),
        serde_json::json!({"$schema":SCHEMA,"rules":{"R10":"advisory"}}).to_string(),
        serde_json::json!({"$schema":SCHEMA,"rules":{"R1":"secret-test-value"}}).to_string(),
        serde_json::json!({"$schema":SCHEMA,"rules":{"R1":{"policy":"off"}}}).to_string(),
        serde_json::json!({"$schema":format!("{SCHEMA}#latest")}).to_string(),
        serde_json::json!({"rules":{"R1":"off"}}).to_string(),
        serde_json::json!({"$schema":SCHEMA,"overrides":[{"include":[],"rules":{}}]}).to_string(),
        serde_json::json!({"$schema":SCHEMA,"extends":"other.jsonc"}).to_string(),
        format!("{{'$schema':'{SCHEMA}'}}"),
        format!("{{\"$schema\":\"{SCHEMA}\" \"rules\":{{}}}}"),
        format!(
            "{{\"$schema\":\"{SCHEMA}\",\"overrides\":[{{\"include\":[1e1000],\"rules\":{{}}}}]}}"
        ),
        "{} true".to_owned(),
        "/* empty */".to_owned(),
    ] {
        install(root.path(), &invalid);
        let error = resolve(root.path(), Path::new("src/file.ts"), None).unwrap_err();
        assert_eq!(error.exit_code(), 2);
        assert!(
            !serde_json::to_string(&error)
                .unwrap()
                .contains("secret-test-value")
        );
        assert!(error.location.unwrap().line.unwrap() >= 1);
    }
}

#[test]
fn explicit_file_and_symlinks_use_the_real_declaring_directory() {
    use std::os::unix::fs::symlink;
    let workspace = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let name = workspace.path().file_name().unwrap().to_str().unwrap();
    let config = external.path().join("config.jsonc");
    fs::write(&config, override_document(&format!("../{name}/src/**"))).unwrap();
    symlink(&config, workspace.path().join("linked.jsonc")).unwrap();
    let snapshot = resolve(
        workspace.path(),
        Path::new("src/future.ts"),
        Some(Path::new("linked.jsonc")),
    )
    .unwrap();
    assert_eq!(snapshot.view().rules[0].policy, Policy::Off);
    match &snapshot.view().source {
        Source::File { file, .. } => assert_eq!(file, &config),
        _ => panic!("file source missing"),
    }
    assert_eq!(snapshot.view().path, "src/future.ts");
    assert!(!workspace.path().join("src").exists());
    assert_eq!(
        resolve(workspace.path(), Path::new("src/future.ts"), Some(&config))
            .unwrap()
            .view(),
        snapshot.view()
    );
}

#[test]
fn missing_explicit_invalid_and_dangling_files_do_not_fall_back() {
    use std::os::unix::fs::symlink;
    let root = TempDir::new().unwrap();
    assert_eq!(
        resolve(
            root.path(),
            Path::new("a.ts"),
            Some(Path::new("missing.jsonc"))
        )
        .unwrap_err()
        .exit_code(),
        3
    );
    symlink("missing-target", root.path().join(".anti-slop.jsonc")).unwrap();
    assert_eq!(
        resolve(root.path(), Path::new("a.ts"), None)
            .unwrap_err()
            .exit_code(),
        3
    );
    fs::remove_file(root.path().join(".anti-slop.jsonc")).unwrap();
    install(root.path(), "broken");
    assert_eq!(
        resolve(root.path(), Path::new("a.ts"), None)
            .unwrap_err()
            .exit_code(),
        2
    );
    for path in [
        "",
        "/outside.ts",
        "../outside.ts",
        "src/../../outside.ts",
        ".",
    ] {
        assert_eq!(
            resolve(root.path(), Path::new(path), None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
    }
}

#[test]
fn captured_values_and_bytes_survive_configuration_edits() {
    let root = TempDir::new().unwrap();
    let original = serde_json::json!({"$schema":SCHEMA,"rules":{"R1":"off"}}).to_string();
    install(root.path(), &original);
    let before = resolve(root.path(), Path::new("a.ts"), None).unwrap();
    install(
        root.path(),
        &serde_json::json!({"$schema":SCHEMA,"rules":{"R1":"block"}}).to_string(),
    );
    let after = resolve(root.path(), Path::new("a.ts"), None).unwrap();
    assert_eq!(before.captured_bytes(), Some(original.as_bytes()));
    assert_eq!(before.view().rules[0].policy, Policy::Off);
    assert_eq!(after.view().rules[0].policy, Policy::Block);
    assert_ne!(before.view().source, after.view().source);
}

#[test]
fn unreadable_or_non_utf8_existing_files_are_errors() {
    use std::os::unix::fs::PermissionsExt;
    let root = TempDir::new().unwrap();
    let file = root.path().join(".anti-slop.jsonc");
    install(
        root.path(),
        &serde_json::json!({"$schema":SCHEMA}).to_string(),
    );
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    assert_eq!(
        resolve(root.path(), Path::new("a.ts"), None)
            .unwrap_err()
            .exit_code(),
        3
    );
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&file, [0xff, 0xfe]).unwrap();
    assert_eq!(
        resolve(root.path(), Path::new("a.ts"), None)
            .unwrap_err()
            .exit_code(),
        2
    );
}

#[test]
fn invalid_fields_have_actionable_locations_and_debug_omits_raw_bytes() {
    let root = TempDir::new().unwrap();
    install(
        root.path(),
        &format!(
            "{{\n  \"$schema\": \"{SCHEMA}\",\n  \"provider\": {{\"api_key\": \"hidden-test-value\"}}\n}}"
        ),
    );
    let error = resolve(root.path(), Path::new("a.ts"), None).unwrap_err();
    assert_eq!(error.location.unwrap().line, Some(3));
    install(
        root.path(),
        &format!(
            "{{\n  \"$schema\": \"{SCHEMA}\",\n  \"rules\": {{\n    \"R10\": \"off\"\n  }}\n}}"
        ),
    );
    let error = resolve(root.path(), Path::new("a.ts"), None).unwrap_err();
    assert_eq!(error.location.unwrap().line, Some(4));
    install(
        root.path(),
        &format!("{{\"$schema\":\"{SCHEMA}\" /* hidden-test-value */}}"),
    );
    let snapshot = resolve(root.path(), Path::new("a.ts"), None).unwrap();
    assert!(!format!("{snapshot:?}").contains("hidden-test-value"));
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("hidden-test-value")
    );
}

#[test]
fn parser_and_matcher_enforce_actual_input_boundaries() {
    let root = TempDir::new().unwrap();
    let mut boundary = serde_json::json!({"$schema":SCHEMA}).to_string();
    boundary.extend(std::iter::repeat_n(' ', MAX_FILE_BYTES - boundary.len()));
    install(root.path(), &boundary);
    assert!(resolve(root.path(), Path::new("a.ts"), None).is_ok());
    boundary.push(' ');
    install(root.path(), &boundary);
    assert_eq!(
        resolve(root.path(), Path::new("a.ts"), None)
            .unwrap_err()
            .code,
        ErrorCode::InputLimit
    );
    for (depth, code) in [
        (63, ErrorCode::InvalidConfiguration),
        (64, ErrorCode::InputLimit),
        (20_000, ErrorCode::InputLimit),
    ] {
        install(
            root.path(),
            &format!(
                "{{\"$schema\":\"{SCHEMA}\",\"unknown\":{}null{}}}",
                "[".repeat(depth),
                "]".repeat(depth)
            ),
        );
        assert_eq!(
            resolve(root.path(), Path::new("a.ts"), None)
                .unwrap_err()
                .code,
            code
        );
    }
    for (count, accepted) in [(256, true), (257, false)] {
        let entries: Vec<_> = (0..count)
            .map(|_| serde_json::json!({"include":["**"],"rules":{}}))
            .collect();
        install(
            root.path(),
            &serde_json::json!({"$schema":SCHEMA,"overrides":entries}).to_string(),
        );
        assert_eq!(
            resolve(root.path(), Path::new("a.ts"), None).is_ok(),
            accepted
        );
    }
    for (count, accepted) in [(1024, true), (1025, false)] {
        let patterns: Vec<_> = (0..count).map(|_| "*.ts").collect();
        install(
            root.path(),
            &serde_json::json!({"$schema":SCHEMA,"overrides":[{"include":patterns,"rules":{}}]})
                .to_string(),
        );
        assert_eq!(
            resolve(root.path(), Path::new("a.ts"), None).is_ok(),
            accepted
        );
    }
    for (size, accepted) in [(4096, true), (4097, false)] {
        install(root.path(), &override_document(&"a".repeat(size)));
        assert_eq!(
            resolve(root.path(), Path::new("a.ts"), None).is_ok(),
            accepted
        );
        install(
            root.path(),
            &serde_json::json!({"$schema":SCHEMA}).to_string(),
        );
        assert_eq!(
            resolve(root.path(), Path::new(&"a".repeat(size)), None).is_ok(),
            accepted
        );
    }
}

#[test]
fn jsonc_ast_preserves_duplicate_decoded_keys_and_comments() {
    let result = jsonc_parser::parse_to_ast(
        "{/* comment */\"R1\":\"off\",\"R\\u0031\":\"block\",}",
        &Default::default(),
        &jsonc_parser::ParseOptions {
            allow_comments: true,
            allow_trailing_commas: true,
            allow_loose_object_property_names: false,
            allow_missing_commas: false,
            allow_single_quoted_strings: false,
            allow_hexadecimal_numbers: false,
            allow_unary_plus_numbers: false,
            allow_bare_decimal_point_numbers: false,
            allow_non_finite_numbers: false,
            allow_extended_string_escapes: false,
        },
    )
    .unwrap();
    let root = result.value.unwrap();
    let object = root.as_object().unwrap();
    assert_eq!(object.properties.len(), 2);
    assert_eq!(object.properties[0].name.as_str(), "R1");
    assert_eq!(object.properties[1].name.as_str(), "R1");
}
