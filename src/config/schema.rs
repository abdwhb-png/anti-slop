use super::{FileConfig, Policy, RuleId, SCHEMA_URL};
use schemars::{Schema, SchemaGenerator, json_schema};

/// Generate the local schema from the same input types that the resolver reads.
pub fn schema_document() -> serde_json::Value {
    let generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
    let schema = generator.into_root_schema_for::<FileConfig>();
    let mut value: serde_json::Value = schema.into();
    value["$id"] = SCHEMA_URL.into();
    value
}

pub(super) fn identity(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "string", "const": SCHEMA_URL })
}

pub(super) fn rules(generator: &mut SchemaGenerator) -> Schema {
    // Enumerate the actual supported rule identities so editor completions use
    // the same inventory as resolution. Policy values derive from the enum.
    let policy: serde_json::Value = generator.subschema_for::<Policy>().into();
    let mut properties = serde_json::Map::new();
    for rule in RuleId::ALL {
        let mut value = policy.clone();
        value["description"] = format!(
            "Requested policy for {}. It does not establish rule capabilities or qualification.",
            rule.as_str()
        )
        .into();
        properties.insert(rule.as_str().into(), value);
    }
    json_schema!({ "type": "object", "properties": properties, "additionalProperties": false })
}
