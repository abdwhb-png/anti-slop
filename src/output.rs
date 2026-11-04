//! Shared configuration inspection projection and explicit encodings.
use crate::config::{ConfigError, EffectiveView};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Format {
    #[default]
    Toon,
    Json,
}
#[derive(Serialize)]
#[serde(untagged)]
pub enum Inspection<'a> {
    Config {
        config: &'a EffectiveView,
    },
    Error {
        error: &'a ConfigError,
        help: &'a str,
    },
}
#[derive(Debug, thiserror::Error)]
#[error("could not encode configuration inspection")]
pub struct EncodeError;

/// TOON 4.1 inspection profile: comma, two spaces, bounded integer metadata,
/// Unicode strings, objects/arrays/null. Both formats consume the same value.
pub fn encode(view: &Inspection<'_>, format: Format) -> Result<String, EncodeError> {
    let value = serde_json::to_value(view).map_err(|_| EncodeError)?;
    match format {
        Format::Toon => serde_toon::to_string_with_options(
            &value,
            &serde_toon::EncodeOptions::new()
                .with_indent(serde_toon::Indent::spaces(2))
                .with_delimiter(serde_toon::Delimiter::Comma),
        )
        .map_err(|_| EncodeError),
        Format::Json => serde_json::to_string_pretty(&value).map_err(|_| EncodeError),
    }
}
