use super::{ConfigError, ErrorCode, FileConfig, Location, MAX_DEPTH, RuleId};
use jsonc_parser::{
    CollectOptions, ParseOptions, Scanner, ScannerOptions, ast, common::Ranged, tokens::Token,
};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub(super) struct Parsed<'a> {
    pub config: FileConfig,
    text: &'a str,
    file: &'a Path,
    offsets: BTreeMap<String, usize>,
}

impl Parsed<'_> {
    pub fn location(&self, pointer: &str) -> Location {
        location(
            self.text,
            self.file,
            *self.offsets.get(pointer).unwrap_or(&0),
        )
    }
    pub fn error(&self, pointer: &str, code: ErrorCode, message: &str) -> ConfigError {
        ConfigError {
            code,
            message: message.to_owned(),
            location: Some(self.location(pointer)),
        }
    }
}

pub(super) fn document<'a>(text: &'a str, file: &'a Path) -> Result<Parsed<'a>, ConfigError> {
    let scanner_options = ScannerOptions {
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    };
    let mut scanner = Scanner::new(text, &scanner_options);
    let mut depth = 0usize;
    while let Some(token) = scanner
        .scan()
        .map_err(|e| syntax_error(text, file, e.range().start))?
    {
        match token {
            Token::OpenBrace | Token::OpenBracket => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err(at(
                        text,
                        file,
                        scanner.token_start(),
                        ErrorCode::InputLimit,
                        "Configuration exceeds nesting depth 64.",
                    ));
                }
            }
            Token::CloseBrace | Token::CloseBracket => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    let options = ParseOptions {
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
    };
    let ast = jsonc_parser::parse_to_ast(text, &CollectOptions::default(), &options)
        .map_err(|e| syntax_error(text, file, e.range().start))?
        .value
        .ok_or_else(|| syntax_error(text, file, 0))?;
    let mut offsets = BTreeMap::new();
    let mut invalid_rule_keys = BTreeMap::new();
    let value = json_value(&ast, "", text, file, &mut offsets, &mut invalid_rule_keys)?;
    let config = serde_path_to_error::deserialize(value).map_err(|error| {
        let pointer = error.path().iter().filter_map(|segment| match segment {
            serde_path_to_error::Segment::Map { key } => Some(escape_pointer(key)),
            serde_path_to_error::Segment::Seq { index } => Some(index.to_string()),
            _ => None,
        }).map(|p| format!("/{p}")).collect::<String>();
        if let Some(offset) = invalid_rule_keys.get(&pointer) {
            return at(text, file, *offset, ErrorCode::InvalidConfiguration, "Unsupported rule identifier; use R1 through R9.");
        }
        at(text, file, *offsets.get(&pointer).unwrap_or(&0), ErrorCode::InvalidConfiguration, "Invalid or unsupported configuration field/value; use the supported schema and scalar rule policies.")
    })?;
    Ok(Parsed {
        config,
        text,
        file,
        offsets,
    })
}

fn json_value(
    value: &ast::Value<'_>,
    pointer: &str,
    text: &str,
    file: &Path,
    offsets: &mut BTreeMap<String, usize>,
    invalid_rule_keys: &mut BTreeMap<String, usize>,
) -> Result<serde_json::Value, ConfigError> {
    offsets.insert(pointer.to_owned(), value.range().start);
    Ok(match value {
        ast::Value::StringLit(v) => serde_json::Value::String(v.value.to_string()),
        ast::Value::BooleanLit(v) => serde_json::Value::Bool(v.value),
        ast::Value::NullKeyword(_) => serde_json::Value::Null,
        ast::Value::NumberLit(_) => {
            return Err(at(
                text,
                file,
                value.range().start,
                ErrorCode::InvalidConfiguration,
                "Numeric configuration values are unsupported.",
            ));
        }
        ast::Value::Array(v) => serde_json::Value::Array(
            v.elements
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    json_value(
                        value,
                        &format!("{pointer}/{i}"),
                        text,
                        file,
                        offsets,
                        invalid_rule_keys,
                    )
                })
                .collect::<Result<_, _>>()?,
        ),
        ast::Value::Object(v) => {
            let mut map = serde_json::Map::new();
            let mut seen = HashSet::new();
            if pointer.ends_with("/rules")
                && let Some(property) = v
                    .properties
                    .iter()
                    .find(|p| !RuleId::ALL.iter().any(|r| r.as_str() == p.name.as_str()))
            {
                invalid_rule_keys.insert(pointer.to_owned(), property.name.range().start);
            }
            for prop in &v.properties {
                let name = prop.name.as_str();
                if !seen.insert(name) {
                    return Err(at(
                        text,
                        file,
                        prop.name.range().start,
                        ErrorCode::InvalidConfiguration,
                        "Duplicate configuration key; remove the repeated property.",
                    ));
                }
                map.insert(
                    name.to_owned(),
                    json_value(
                        &prop.value,
                        &format!("{pointer}/{}", escape_pointer(name)),
                        text,
                        file,
                        offsets,
                        invalid_rule_keys,
                    )?,
                );
            }
            serde_json::Value::Object(map)
        }
    })
}

fn escape_pointer(text: &str) -> String {
    text.replace('~', "~0").replace('/', "~1")
}

fn location(text: &str, file: &Path, offset: usize) -> Location {
    let prefix = &text[..offset.min(text.len())];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    Location {
        file: file.to_owned(),
        line: Some(line),
        column: Some(column),
    }
}

fn at(text: &str, file: &Path, offset: usize, code: ErrorCode, message: &str) -> ConfigError {
    ConfigError {
        code,
        message: message.to_owned(),
        location: Some(location(text, file, offset)),
    }
}

fn syntax_error(text: &str, file: &Path, offset: usize) -> ConfigError {
    at(
        text,
        file,
        offset,
        ErrorCode::InvalidConfiguration,
        "Invalid JSONC syntax; use quoted keys and supported values.",
    )
}
