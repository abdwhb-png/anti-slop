#[test]
fn generated_schema_requires_the_exact_runtime_identity() {
    let schema = anti_slop::config::schema_document();
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["$id"], anti_slop::config::SCHEMA_URL);
    assert_eq!(
        schema["properties"]["$schema"]["const"],
        anti_slop::config::SCHEMA_URL
    );
}

#[test]
fn structural_conformance_fixtures_pass_through_the_actual_resolver() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/config/conformance.json")).unwrap();
    let root = tempfile::TempDir::new().unwrap();
    for case in cases.as_array().unwrap() {
        std::fs::write(
            root.path().join(".anti-slop.jsonc"),
            case["document"].to_string(),
        )
        .unwrap();
        let result =
            anti_slop::config::resolve(root.path(), std::path::Path::new("src/future.ts"), None);
        assert_eq!(
            result.is_ok(),
            case["valid"].as_bool().unwrap(),
            "{}: {result:?}",
            case["name"]
        );
    }
}

#[test]
fn regeneration_is_deterministic_and_local_artifact_matches_the_real_owner() {
    let first = serde_json::to_string_pretty(&anti_slop::config::schema_document()).unwrap() + "\n";
    let second =
        serde_json::to_string_pretty(&anti_slop::config::schema_document()).unwrap() + "\n";
    assert_eq!(first, second);
    assert_eq!(first, include_str!("../schemas/anti-slop.schema.json"));
}
