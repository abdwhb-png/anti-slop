//! Retained HEAD-to-index changes and independently observed working-file context.
//! Acquisition checks two observations of relevant inputs; it does not freeze the filesystem.
mod acquire;
mod git;

use crate::config;
use acquire::Acquisition;
use git::Git;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ops::Range,
    path::{Path, PathBuf},
    time::Duration,
};

pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_FILES: usize = 512;
pub const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_INVENTORY_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_ATTEMPTS: usize = 2;
pub const GIT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitMode {
    Regular,
    Executable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Head,
    Index,
    Context,
    Configuration,
}

pub struct Source {
    pub path: String,
    pub role: Role,
    pub generation: String,
    pub git_blob: Option<String>,
    pub git_mode: Option<GitMode>,
    bytes: Vec<u8>,
}
impl Source {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn new(
        path: String,
        role: Role,
        git_blob: Option<String>,
        git_mode: Option<GitMode>,
        bytes: Vec<u8>,
    ) -> Self {
        let generation = identity(&[
            format!("{role:?}:{git_mode:?}").as_bytes(),
            path.as_bytes(),
            git_blob.as_deref().unwrap_or("").as_bytes(),
            &bytes,
        ]);
        Self {
            path,
            role,
            generation,
            git_blob,
            git_mode,
            bytes,
        }
    }
}
impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("path", &self.path)
            .field("role", &self.role)
            .field("generation", &self.generation)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct Change {
    pub path: String,
    pub before: Option<Source>,
    pub after: Option<Source>,
}

#[derive(Debug)]
pub struct Capture {
    head: String,
    generation: String,
    changes: Vec<Change>,
    context: Vec<Source>,
    configuration: config::Configuration,
    stats: CaptureStats,
}

#[derive(Debug)]
pub struct CaptureStats {
    pub retained_files: usize,
    pub retained_bytes: usize,
    pub attempts: usize,
    pub git_processes: usize,
}

/// Calling this entry point explicitly chooses staged selection. No other selector is implemented.
#[derive(Debug, Default)]
pub struct StagedRequest {
    pub context: Vec<PathBuf>,
    pub exclusions: Vec<PathBuf>,
    pub configuration: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedInput {
    pub path: String,
    pub role: Role,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Invalid capture input: {0}")]
    InvalidInput(String),
    #[error("Required input unavailable: {0}")]
    Unavailable(String),
    #[error("Excluded required inputs: {0:?}")]
    Excluded(Vec<ObservedInput>),
    #[error("Limit exceeded: {0}")]
    Limit(&'static str),
    #[error("Unsupported input: {0}")]
    Unsupported(&'static str),
    #[error("Required inputs changed during both acquisition attempts")]
    Unstable,
    #[error("Git exceeded its process deadline")]
    Timeout,
    #[error(transparent)]
    Configuration(#[from] config::ConfigError),
}

/// Byte offsets, including empty/EOF ranges. A range must bind the exact retained path/role/generation.
#[derive(Debug, Clone)]
pub struct SourceRange {
    pub path: String,
    pub role: Role,
    pub generation: String,
    pub range: Range<usize>,
}
impl Capture {
    pub fn head(&self) -> &str {
        &self.head
    }
    pub fn generation(&self) -> &str {
        &self.generation
    }
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }
    pub fn context(&self) -> &[Source] {
        &self.context
    }
    pub fn configuration(&self) -> &config::Configuration {
        &self.configuration
    }
    pub fn stats(&self) -> &CaptureStats {
        &self.stats
    }

    pub fn slice(&self, range: &SourceRange) -> Result<&[u8], CaptureError> {
        self.changes
            .iter()
            .flat_map(|c| c.before.iter().chain(c.after.iter()))
            .chain(self.context.iter())
            .find(|s| {
                s.path == range.path && s.role == range.role && s.generation == range.generation
            })
            .and_then(|s| s.bytes.get(range.range.clone()))
            .ok_or_else(|| CaptureError::InvalidInput("Invalid or unbound byte range".into()))
    }
}

struct Entry {
    path: String,
    old: Option<String>,
    new: Option<String>,
    old_mode: Option<GitMode>,
    new_mode: Option<GitMode>,
}

/// Capture is read-only. An error returns no partial source bytes or replacement source view.
pub fn capture_staged(workspace: &Path, request: &StagedRequest) -> Result<Capture, CaptureError> {
    let root = config::normalize_workspace(workspace)?;
    if request
        .configuration
        .as_ref()
        .is_some_and(|p| p.as_os_str().len() > config::MAX_PATH_BYTES)
    {
        return Err(CaptureError::Limit("Configuration path bytes"));
    }
    let contexts = paths(&request.context)?;
    let exclusions = paths(&request.exclusions)?;
    let mut git = Git { processes: 0 };
    let check = Acquisition::new(&root)?;
    let top = git.read(
        &check.cwd(),
        &["rev-parse", "--show-toplevel"],
        config::MAX_PATH_BYTES + 1,
    )?;
    // rev-parse emits one terminal newline; path-internal newlines remain part of the identity.
    if top.strip_suffix(b"\n") != Some(root.as_os_str().as_encoded_bytes()) {
        return Err(CaptureError::InvalidInput(
            "Workspace must be the repository root".into(),
        ));
    }
    let head = git.read(
        &check.cwd(),
        &["rev-parse", "--verify", "HEAD^{commit}"],
        65,
    )?;
    let head = std::str::from_utf8(head.strip_suffix(b"\n").unwrap_or(&head))
        .map_err(|_| CaptureError::InvalidInput("Invalid HEAD identity".into()))?
        .to_owned();
    if !oid(&head) {
        return Err(CaptureError::InvalidInput("Invalid HEAD identity".into()));
    }
    for attempt in 1..=MAX_ATTEMPTS {
        let mut acquire = Acquisition::new(&root)?;
        let cwd = acquire.cwd();
        let inventory = git.inventory(&cwd, &head)?;
        let entries = inventory_entries(&inventory)?;
        let config_path = config::selected_path(&root, request.configuration.as_deref())?;
        let mut excluded = Vec::new();
        for entry in &entries {
            if exclusions.contains(&entry.path) {
                if entry.old.is_some() {
                    excluded.push(ObservedInput {
                        path: entry.path.clone(),
                        role: Role::Head,
                    });
                }
                if entry.new.is_some() {
                    excluded.push(ObservedInput {
                        path: entry.path.clone(),
                        role: Role::Index,
                    });
                }
            }
        }
        for path in &contexts {
            if exclusions.contains(path) {
                excluded.push(ObservedInput {
                    path: path.clone(),
                    role: Role::Context,
                });
            }
        }
        check_configuration_exclusion(&root, &config_path, &exclusions, &mut excluded)?;
        if !excluded.is_empty() {
            return Err(CaptureError::Excluded(excluded));
        }
        let file = config::locate(&root, request.configuration.as_deref())?;
        if let Some(file) = &file {
            check_configuration_exclusion(&root, file, &exclusions, &mut excluded)?;
        }
        if !excluded.is_empty() {
            return Err(CaptureError::Excluded(excluded));
        }
        let config_bytes = if let Some(file) = &file {
            acquire.admit_file()?;
            let bytes = acquire.configuration(file, acquire.remaining())?;
            acquire.admit_bytes(bytes.len())?;
            Some(bytes)
        } else {
            None
        };
        let configuration = config::from_captured(root.clone(), file.clone(), config_bytes)?;
        let mut changes = Vec::new();
        for entry in entries {
            let mut side =
                |blob: Option<String>, mode, role| -> Result<Option<Source>, CaptureError> {
                    let Some(blob) = blob else {
                        return Ok(None);
                    };
                    acquire.admit_file()?;
                    // cat-file reads the requested object verbatim: no checkout, attribute filters or textconv.
                    let bytes = git.blob(&cwd, &blob, acquire.remaining())?;
                    acquire.admit_bytes(bytes.len())?;
                    Ok(Some(Source::new(
                        entry.path.clone(),
                        role,
                        Some(blob),
                        mode,
                        bytes,
                    )))
                };
            let before = side(entry.old, entry.old_mode, Role::Head)?;
            let after = side(entry.new, entry.new_mode, Role::Index)?;
            changes.push(Change {
                path: entry.path,
                before,
                after,
            });
        }
        let mut context = Vec::new();
        for path in &contexts {
            acquire.admit_file()?;
            let bytes = acquire.working(path, acquire.remaining())?;
            acquire.admit_bytes(bytes.len())?;
            context.push(Source::new(path.clone(), Role::Context, None, None, bytes));
        }
        // Compare actual bytes and a fresh relevant index inventory, not timestamps. This is an
        // observational check, not a global atomic snapshot; ABA edits can remain unobserved.
        let mut stable = inventory == git.inventory(&cwd, &head)?;
        for source in &context {
            stable &= source.bytes == acquire.working(&source.path, MAX_FILE_BYTES)?;
        }
        let current_file = config::locate(&root, request.configuration.as_deref())?;
        stable &= file == current_file;
        if let Some(file) = &current_file {
            check_configuration_exclusion(&root, file, &exclusions, &mut excluded)?;
            if !excluded.is_empty() {
                return Err(CaptureError::Excluded(excluded));
            }
            stable &= configuration.captured_bytes()
                == Some(acquire.configuration(file, MAX_FILE_BYTES)?.as_slice());
        }
        if !stable {
            continue;
        }
        let mut generations = vec![head.as_bytes(), configuration.generation().as_bytes()];
        generations.extend(
            changes
                .iter()
                .flat_map(|c| c.before.iter().chain(c.after.iter()))
                .chain(context.iter())
                .map(|s| s.generation.as_bytes()),
        );
        let generation = identity(&generations);
        return Ok(Capture {
            head,
            generation,
            changes,
            context,
            configuration,
            stats: CaptureStats {
                retained_files: acquire.files,
                retained_bytes: acquire.bytes,
                attempts: attempt,
                git_processes: git.processes,
            },
        });
    }
    Err(CaptureError::Unstable)
}

fn paths(inputs: &[PathBuf]) -> Result<BTreeSet<String>, CaptureError> {
    if inputs.len() > MAX_FILES {
        return Err(CaptureError::Limit("Requested path count"));
    }
    inputs
        .iter()
        .map(|p| config::normalize_path(p).map_err(CaptureError::from))
        .collect()
}
fn check_configuration_exclusion(
    root: &Path,
    file: &Path,
    exclusions: &BTreeSet<String>,
    observed: &mut Vec<ObservedInput>,
) -> Result<(), CaptureError> {
    let mut absolute = PathBuf::new();
    for component in file.components() {
        match component {
            std::path::Component::ParentDir => {
                absolute.pop();
            }
            std::path::Component::CurDir => {}
            _ => absolute.push(component.as_os_str()),
        }
    }
    if let Ok(relative) = absolute.strip_prefix(root) {
        let path = config::normalize_path(relative)?;
        if exclusions.contains(&path) {
            observed.push(ObservedInput {
                path,
                role: Role::Configuration,
            });
        }
    }
    Ok(())
}
fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn inventory_entries(inventory: &[u8]) -> Result<Vec<Entry>, CaptureError> {
    if !inventory.is_empty() && !inventory.ends_with(&[0]) {
        return Err(CaptureError::Unsupported("Truncated Git inventory"));
    }
    let mut fields = inventory.split(|b| *b == 0);
    let mut entries = Vec::new();
    while let Some(header) = fields.next().filter(|s| !s.is_empty()) {
        if entries.len() >= MAX_FILES {
            return Err(CaptureError::Limit("Selected path count"));
        }
        let header = std::str::from_utf8(header)
            .map_err(|_| CaptureError::Unsupported("Non-UTF-8 inventory"))?;
        let mut split = header.split(' ');
        let parts = std::array::from_fn::<_, 5, _>(|_| split.next().unwrap_or(""));
        if split.next().is_some() || !parts[0].starts_with(':') || !oid(parts[2]) || !oid(parts[3])
        {
            return Err(CaptureError::Unsupported("Malformed Git inventory"));
        }
        if !matches!(parts[4], "A" | "D" | "M" | "T") {
            return Err(CaptureError::Unsupported(
                "Unmerged or unsupported index state",
            ));
        }
        for mode in [&parts[0][1..], parts[1]] {
            if !matches!(mode, "000000" | "100644" | "100755") {
                return Err(CaptureError::Unsupported(
                    "Only regular Git blobs are supported",
                ));
            }
        }
        let path = fields
            .next()
            .ok_or(CaptureError::Unsupported("Missing Git path"))?;
        let path = std::str::from_utf8(path)
            .map_err(|_| CaptureError::Unsupported("Non-UTF-8 source path"))?;
        let normalized = config::normalize_path(Path::new(path))?;
        if normalized != path {
            return Err(CaptureError::Unsupported("Noncanonical Git path"));
        }
        let side = |s: &str| {
            if s.bytes().all(|b| b == b'0') {
                None
            } else {
                Some(s.to_owned())
            }
        };
        let mode = |s| match s {
            "100644" => Some(GitMode::Regular),
            "100755" => Some(GitMode::Executable),
            _ => None,
        };
        entries.push(Entry {
            path: normalized,
            old: side(parts[2]),
            new: side(parts[3]),
            old_mode: mode(&parts[0][1..]),
            new_mode: mode(parts[1]),
        });
    }
    Ok(entries)
}
fn identity(parts: &[&[u8]]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_le_bytes());
        digest.update(part);
    }
    digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
