//! Offline root configuration resolution shared by application entrypoints.

mod parse;
mod pattern;
mod schema;

pub use schema::schema_document;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

pub const CONTRACT_ID: &str = "config-v1.0.0";
pub const SCHEMA_URL: &str =
    "https://github.com/abdwhb-png/anti-slop/releases/download/config-v1.0.0/anti-slop.schema.json";
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_DEPTH: usize = 64;
pub const MAX_OVERRIDES: usize = 256;
pub const MAX_PATTERNS: usize = 1024;
pub const MAX_PATH_BYTES: usize = 4096;

/// Recognized configuration identities, not an assertion of implemented rule capabilities.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, schemars::JsonSchema,
)]
pub enum RuleId {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
}

impl RuleId {
    pub const ALL: [Self; 9] = [
        Self::R1,
        Self::R2,
        Self::R3,
        Self::R4,
        Self::R5,
        Self::R6,
        Self::R7,
        Self::R8,
        Self::R9,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::R1 => "R1",
            Self::R2 => "R2",
            Self::R3 => "R3",
            Self::R4 => "R4",
            Self::R5 => "R5",
            Self::R6 => "R6",
            Self::R7 => "R7",
            Self::R8 => "R8",
            Self::R9 => "R9",
        }
    }
}

/// Requested policy; `block` does not establish qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Policy {
    /// Request no evaluation of this rule.
    Off,
    /// Request evaluation without blocking; the default for unspecified root rules.
    Advisory,
    /// Request blocking, subject to separate scope and qualification requirements.
    Block,
}

/// Local configuration contract. It grants neither analysis scope nor transmission authorization.
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Anti-Slop local configuration")]
pub(crate) struct FileConfig {
    /// Exact supported contract identity. This reserved release asset is unpublished; validation is offline.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "schema::identity")]
    pub schema: String,
    /// Root requested policies. Unspecified rules use built-in advisory defaults.
    #[serde(default)]
    #[schemars(schema_with = "schema::rules")]
    pub rules: BTreeMap<RuleId, Policy>,
    /// Matching overrides apply in declaration order, replacing assigned whole policies.
    #[serde(default)]
    #[schemars(length(max = MAX_OVERRIDES))]
    pub overrides: Vec<Override>,
}

/// One self-contained path override; exclusions affect this override only.
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Override {
    /// Any include must match. Case-sensitive globs relative to the real declaring file; grammar and UTF-8 byte limits are checked at runtime.
    #[schemars(length(min = 1, max = MAX_PATTERNS), inner(length(min = 1, max = MAX_PATH_BYTES)))]
    pub include: Vec<String>,
    /// Any exclude prevents only this override. All include/exclude patterns combined are limited to 1,024 at runtime.
    #[serde(default)]
    #[schemars(length(max = MAX_PATTERNS), inner(length(min = 1, max = MAX_PATH_BYTES)))]
    pub exclude: Vec<String>,
    /// Replace assigned policies; omitted rules preserve earlier root/override values.
    #[schemars(schema_with = "schema::rules")]
    pub rules: BTreeMap<RuleId, Policy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Origin {
    BuiltIn {
        contract: String,
    },
    Root {
        pointer: String,
        location: Location,
    },
    Override {
        index: usize,
        pointer: String,
        location: Location,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleSetting {
    pub rule: RuleId,
    pub policy: Policy,
    pub origin: Origin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    BuiltIn,
    File {
        file: PathBuf,
        sha256: String,
        bytes: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveView {
    pub workspace: PathBuf,
    pub path: String,
    pub contract: String,
    pub schema: String,
    pub source: Source,
    pub matched_overrides: Vec<usize>,
    pub rules: Vec<RuleSetting>,
}

/// Owned immutable inputs; a later load cannot mutate a previously returned snapshot.
#[derive(Serialize)]
pub struct Snapshot {
    #[serde(flatten)]
    view: EffectiveView,
    #[serde(skip)]
    captured: Option<Arc<[u8]>>,
}

impl Snapshot {
    pub fn view(&self) -> &EffectiveView {
        &self.view
    }
    pub fn captured_bytes(&self) -> Option<&[u8]> {
        self.captured.as_deref()
    }
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshot")
            .field("view", &self.view)
            .field("captured_bytes", &self.captured.as_ref().map(|b| b.len()))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    InvalidConfiguration,
    InvalidPattern,
    InputLimit,
    Io,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigError {
    pub code: ErrorCode,
    pub message: String,
    pub location: Option<Location>,
}

impl ConfigError {
    pub fn exit_code(&self) -> i32 {
        if self.code == ErrorCode::Io { 3 } else { 2 }
    }

    pub(crate) fn new(code: ErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_owned(),
            location: None,
        }
    }

    pub(crate) fn at_file(mut self, file: &Path) -> Self {
        self.location = Some(Location {
            file: file.to_owned(),
            line: None,
            column: None,
        });
        self
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for ConfigError {}

/// Resolve configuration for a lexical workspace-relative path without reading that source path.
pub fn resolve(
    workspace: &Path,
    path: &Path,
    explicit: Option<&Path>,
) -> Result<Snapshot, ConfigError> {
    let workspace = fs::canonicalize(workspace).map_err(|_| {
        ConfigError::new(
            ErrorCode::InvalidInput,
            "Workspace must be an existing readable directory.",
        )
    })?;
    if !workspace.is_dir() || workspace.to_str().is_none() {
        return Err(ConfigError::new(
            ErrorCode::InvalidInput,
            "Workspace must be an existing UTF-8 directory.",
        ));
    }
    let relative = normalize_path(path)?;
    let target = workspace.join(&relative);
    let mut snapshot = Snapshot {
        view: EffectiveView {
            workspace: workspace.clone(),
            path: relative,
            contract: CONTRACT_ID.to_owned(),
            schema: SCHEMA_URL.to_owned(),
            source: Source::BuiltIn,
            matched_overrides: Vec::new(),
            rules: RuleId::ALL
                .into_iter()
                .map(|rule| RuleSetting {
                    rule,
                    policy: Policy::Advisory,
                    origin: Origin::BuiltIn {
                        contract: CONTRACT_ID.to_owned(),
                    },
                })
                .collect(),
        },
        captured: None,
    };
    let selected = explicit.map_or_else(
        || workspace.join(".anti-slop.jsonc"),
        |p| {
            if p.is_absolute() {
                p.to_owned()
            } else {
                workspace.join(p)
            }
        },
    );
    if selected.to_str().is_none() {
        return Err(ConfigError::new(
            ErrorCode::InvalidInput,
            "Configuration paths must be UTF-8.",
        ));
    }
    match fs::symlink_metadata(&selected) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && explicit.is_none() => {
            return Ok(snapshot);
        }
        Err(_) => {
            return Err(ConfigError::new(
                ErrorCode::Io,
                "Cannot access the selected configuration file.",
            )
            .at_file(&selected));
        }
        Ok(_) => {}
    }
    let file = fs::canonicalize(&selected).map_err(|_| {
        ConfigError::new(
            ErrorCode::Io,
            "Cannot resolve the selected configuration file.",
        )
        .at_file(&selected)
    })?;
    if file.to_str().is_none() {
        return Err(ConfigError::new(
            ErrorCode::InvalidInput,
            "Resolved configuration paths must be UTF-8.",
        ));
    }
    if !fs::metadata(&file).is_ok_and(|m| m.is_file()) {
        return Err(ConfigError::new(
            ErrorCode::Io,
            "Configuration must be a readable regular file.",
        )
        .at_file(&file));
    }
    let mut bytes = Vec::new();
    File::open(&file)
        .and_then(|f| f.take((MAX_FILE_BYTES + 1) as u64).read_to_end(&mut bytes))
        .map_err(|_| {
            ConfigError::new(
                ErrorCode::Io,
                "Cannot read the selected configuration file.",
            )
            .at_file(&file)
        })?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(ConfigError::new(
            ErrorCode::InputLimit,
            "Configuration exceeds the 1 MiB limit.",
        )
        .at_file(&file));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        ConfigError::new(
            ErrorCode::InvalidConfiguration,
            "Configuration must be valid UTF-8.",
        )
        .at_file(&file)
    })?;
    let parsed = parse::document(text, &file)?;
    let config = &parsed.config;
    if config.schema != SCHEMA_URL {
        return Err(parsed.error(
            "/$schema",
            ErrorCode::InvalidConfiguration,
            "Unsupported schema identity; use the supported version-pinned schema.",
        ));
    }
    if config.overrides.len() > MAX_OVERRIDES {
        return Err(parsed.error(
            "/overrides",
            ErrorCode::InputLimit,
            "Configuration exceeds 256 overrides.",
        ));
    }
    let count: usize = config
        .overrides
        .iter()
        .map(|o| o.include.len() + o.exclude.len())
        .sum();
    if count > MAX_PATTERNS {
        return Err(parsed.error(
            "/overrides",
            ErrorCode::InputLimit,
            "Configuration exceeds 1024 include/exclude patterns.",
        ));
    }
    for (rule, policy) in &config.rules {
        let pointer = format!("/rules/{}", rule.as_str());
        let setting = &mut snapshot.view.rules[*rule as usize];
        setting.policy = *policy;
        setting.origin = Origin::Root {
            location: parsed.location(&pointer),
            pointer,
        };
    }
    let base = file
        .parent()
        .expect("absolute configuration path has a parent");
    let matched_path = relative_to(&target, base);
    for (index, item) in config.overrides.iter().enumerate() {
        let pointer = format!("/overrides/{index}");
        if item.include.is_empty() {
            return Err(parsed.error(
                &format!("{pointer}/include"),
                ErrorCode::InvalidConfiguration,
                "Override include must contain at least one pattern.",
            ));
        }
        let include = compile_patterns(&item.include, &format!("{pointer}/include"), &parsed)?;
        let exclude = compile_patterns(&item.exclude, &format!("{pointer}/exclude"), &parsed)?;
        if include.iter().any(|p| p.is_match(&matched_path))
            && !exclude.iter().any(|p| p.is_match(&matched_path))
        {
            snapshot.view.matched_overrides.push(index);
            for (rule, policy) in &item.rules {
                let pointer = format!("{pointer}/rules/{}", rule.as_str());
                let setting = &mut snapshot.view.rules[*rule as usize];
                setting.policy = *policy;
                setting.origin = Origin::Override {
                    index,
                    location: parsed.location(&pointer),
                    pointer,
                };
            }
        }
    }
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    snapshot.view.source = Source::File {
        file,
        sha256: digest,
        bytes: bytes.len(),
    };
    snapshot.captured = Some(bytes.into());
    Ok(snapshot)
}

fn compile_patterns(
    patterns: &[String],
    pointer: &str,
    parsed: &parse::Parsed<'_>,
) -> Result<Vec<pattern::Matcher>, ConfigError> {
    patterns
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let pointer = format!("{pointer}/{index}");
            if input.len() > MAX_PATH_BYTES {
                return Err(parsed.error(
                    &pointer,
                    ErrorCode::InputLimit,
                    "Pattern exceeds 4096 UTF-8 bytes.",
                ));
            }
            pattern::Matcher::new(input).map_err(|_| {
                parsed.error(
                    &pointer,
                    ErrorCode::InvalidPattern,
                    "Malformed or unsupported glob; use the documented include/exclude syntax.",
                )
            })
        })
        .collect()
}

fn normalize_path(path: &Path) -> Result<String, ConfigError> {
    let invalid = || {
        ConfigError::new(
            ErrorCode::InvalidInput,
            "Path must be nonempty, UTF-8 and relative within the selected workspace.",
        )
    };
    let text = path.to_str().ok_or_else(invalid)?;
    if text.is_empty() || text.contains('\0') || text.len() > MAX_PATH_BYTES {
        return Err(invalid());
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir if normalized.pop() => {}
            _ => return Err(invalid()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(invalid());
    }
    normalized
        .into_os_string()
        .into_string()
        .map_err(|_| invalid())
}

fn relative_to(target: &Path, base: &Path) -> String {
    let target: Vec<_> = target.components().collect();
    let base: Vec<_> = base.components().collect();
    let common = target.iter().zip(&base).take_while(|(a, b)| a == b).count();
    let mut relative = PathBuf::new();
    for _ in &base[common..] {
        relative.push("..");
    }
    for component in &target[common..] {
        relative.push(component.as_os_str());
    }
    relative.to_str().expect("validated UTF-8 paths").to_owned()
}
