//! Explicit Git comparisons and independently attributable retained working-file context.
//! Acquisition checks two observations of relevant inputs; it does not freeze the filesystem.
mod acquire;
mod git;

use crate::config;
use acquire::{Acquisition, WorkingFile};
use git::Git;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
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
    Base,
    Index,
    Working,
    Context,
    Configuration,
}

pub struct Source {
    pub path: String,
    pub role: Role,
    pub generation: String,
    pub git_blob: Option<String>,
    pub git_mode: Option<GitMode>,
    bytes: Arc<[u8]>,
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
        bytes: Arc<[u8]>,
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

/// Differing before/after paths denote a unique exact-content-and-mode attribution.
/// Indistinguishable deletion/addition groups remain unpaired; physical move intent is not proven.
#[derive(Debug)]
pub struct Change {
    pub path: String,
    pub before: Option<Source>,
    pub after: Option<Source>,
}

#[derive(Debug)]
pub struct Capture {
    comparison: Comparison,
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
    pub acquired_files: usize,
    pub acquired_bytes: usize,
    pub attempts: usize,
    pub git_processes: usize,
}

/// One explicit comparison; missing/conflicting selectors are unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Staged,
    Unstaged,
    Local,
    Base(String),
}

#[derive(Debug)]
pub struct CaptureRequest {
    pub selection: Selection,
    pub context: Vec<PathBuf>,
    pub exclusions: Vec<PathBuf>,
    pub configuration: Option<PathBuf>,
}

impl CaptureRequest {
    pub fn new(selection: Selection) -> Self {
        Self {
            selection,
            context: Vec::new(),
            exclusions: Vec::new(),
            configuration: None,
        }
    }
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
    pub fn selection(&self) -> &Selection {
        &self.comparison.selection
    }
    pub fn resolved_commit(&self) -> Option<&str> {
        self.comparison.commit.as_deref()
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

#[derive(Debug)]
struct Comparison {
    selection: Selection,
    commit: Option<String>,
    before_role: Role,
    after_role: Role,
    command: Vec<String>,
}

fn resolve_comparison(
    git: &mut Git,
    cwd: &Path,
    selection: &Selection,
) -> Result<Comparison, CaptureError> {
    let (reference, before_role, after_role) = match selection {
        Selection::Staged => (Some("HEAD"), Role::Head, Role::Index),
        Selection::Unstaged => (None, Role::Index, Role::Working),
        Selection::Local => (Some("HEAD"), Role::Head, Role::Working),
        Selection::Base(reference) => (Some(reference.as_str()), Role::Base, Role::Working),
    };
    let commit = if let Some(reference) = reference {
        if reference.is_empty()
            || reference.len() > config::MAX_PATH_BYTES
            || reference.contains('\0')
        {
            return Err(CaptureError::InvalidInput(
                "Base must be a nonempty bounded Git reference".into(),
            ));
        }
        let bytes = git.read(
            cwd,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{reference}^{{commit}}"),
            ],
            65,
        )?;
        let value = std::str::from_utf8(bytes.strip_suffix(b"\n").unwrap_or(&bytes))
            .map_err(|_| CaptureError::InvalidInput("Invalid commit identity".into()))?
            .to_owned();
        if !oid(&value) {
            return Err(CaptureError::InvalidInput("Invalid commit identity".into()));
        }
        Some(value)
    } else {
        None
    };
    let empty_tree = if after_role == Role::Working {
        let bytes = git.read(cwd, &["hash-object", "-t", "tree", "--stdin"], 65)?;
        let value = std::str::from_utf8(bytes.strip_suffix(b"\n").unwrap_or(&bytes))
            .map_err(|_| CaptureError::Unsupported("Invalid empty-tree identity"))?
            .to_owned();
        if !oid(&value) {
            return Err(CaptureError::Unsupported("Invalid empty-tree identity"));
        }
        Some(value)
    } else {
        None
    };
    // Cached index/tree inventories never compare filesystem content inside Git. Even raw
    // diff-files/diff-index can execute clean filters for racy unchanged index entries.
    let mut command: Vec<String> = if after_role == Role::Index || before_role == Role::Index {
        vec!["diff-index".into(), "--cached".into()]
    } else {
        vec!["diff-tree".into(), "-r".into(), "--no-commit-id".into()]
    };
    command.extend(
        [
            "--raw",
            "--ita-invisible-in-index",
            "--no-renames",
            "--no-relative",
            "--no-ext-diff",
            "--no-textconv",
            "--ignore-submodules=none",
            "--abbrev=64",
            "-z",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    if let Some(empty) = &empty_tree {
        command.push(empty.clone());
    }
    if let Some(commit) = &commit {
        command.push(commit.clone());
    }
    command.push("--".into());
    Ok(Comparison {
        selection: selection.clone(),
        commit,
        before_role,
        after_role,
        command,
    })
}

/// Capture is read-only. An error returns no partial source bytes or replacement source view.
pub fn capture(workspace: &Path, request: &CaptureRequest) -> Result<Capture, CaptureError> {
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
    if top.strip_suffix(b"\n") != Some(root.as_os_str().as_encoded_bytes()) {
        return Err(CaptureError::InvalidInput(
            "Workspace must be the repository root".into(),
        ));
    }
    let comparison = resolve_comparison(&mut git, &check.cwd(), &request.selection)?;
    let command: Vec<_> = comparison.command.iter().map(String::as_str).collect();
    let untracked_command = ["ls-files", "--others", "--exclude-standard", "-z"];
    let working_selection = comparison.after_role == Role::Working;
    for attempt in 1..=MAX_ATTEMPTS {
        let mut acquire = Acquisition::new(&root)?;
        let cwd = acquire.cwd();
        let (index_flags, index_paths) = if working_selection {
            let flags = git.read(&cwd, &["ls-files", "-v", "-z"], MAX_INVENTORY_BYTES)?;
            let paths = validate_index_flags(&flags)?;
            (flags, paths)
        } else {
            (Vec::new(), Vec::new())
        };
        let inventory = git.read(&cwd, &command, MAX_INVENTORY_BYTES)?;
        let mut baseline = inventory_entries(&inventory)?;
        if working_selection {
            for entry in &mut baseline {
                entry.old = entry.new.take();
                entry.old_mode = entry.new_mode.take();
            }
        }
        let mut entries: BTreeMap<_, _> =
            baseline.into_iter().map(|e| (e.path.clone(), e)).collect();
        let untracked = if working_selection {
            git.read(&cwd, &untracked_command, MAX_INVENTORY_BYTES)?
        } else {
            Vec::new()
        };
        for path in index_paths.into_iter().chain(inventory_paths(&untracked)?) {
            entries.entry(path.clone()).or_insert(Entry {
                path,
                old: None,
                new: None,
                old_mode: None,
                new_mode: None,
            });
        }
        if entries.len() > MAX_FILES {
            return Err(CaptureError::Limit("Selected path count"));
        }
        let entries: Vec<_> = entries.into_values().collect();
        let config_path = config::selected_path(&root, request.configuration.as_deref())?;
        let mut excluded = Vec::new();
        for entry in &entries {
            if exclusions.contains(&entry.path) {
                if entry.old.is_some() {
                    excluded.push(ObservedInput {
                        path: entry.path.clone(),
                        role: comparison.before_role,
                    });
                }
                if working_selection || entry.new.is_some() {
                    excluded.push(ObservedInput {
                        path: entry.path.clone(),
                        role: comparison.after_role,
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
        let mut working = BTreeMap::new();
        for entry in entries {
            let mut stored =
                |blob: Option<String>, mode, role| -> Result<Option<Source>, CaptureError> {
                    let Some(blob) = blob else {
                        return Ok(None);
                    };
                    acquire.admit_file()?;
                    let bytes = git.blob(&cwd, &blob, acquire.remaining())?;
                    acquire.admit_bytes(bytes.len())?;
                    Ok(Some(Source::new(
                        entry.path.clone(),
                        role,
                        Some(blob),
                        mode,
                        bytes.into(),
                    )))
                };
            let before = stored(entry.old, entry.old_mode, comparison.before_role)?;
            let after = if working_selection {
                let file = load_working(&mut acquire, &mut working, &entry.path)?;
                file.map(|f| {
                    Source::new(
                        entry.path.clone(),
                        Role::Working,
                        None,
                        Some(f.mode),
                        f.bytes,
                    )
                })
            } else {
                stored(entry.new, entry.new_mode, comparison.after_role)?
            };
            let unchanged = match (&before, &after) {
                (Some(a), Some(b)) => a.bytes == b.bytes && a.git_mode == b.git_mode,
                (None, None) => true,
                _ => false,
            };
            if !unchanged {
                changes.push(Change {
                    path: entry.path,
                    before,
                    after,
                });
            }
        }
        let mut context = Vec::new();
        for path in &contexts {
            let file = load_working(&mut acquire, &mut working, path)?.ok_or_else(|| {
                CaptureError::Unavailable(format!("Required context is missing: {path}"))
            })?;
            context.push(Source::new(
                path.clone(),
                Role::Context,
                None,
                Some(file.mode),
                file.bytes,
            ));
        }
        // Relevant inventories and exact mutable bytes are checked again. This is not a
        // globally atomic snapshot: edits that revert between observations may be unobserved.
        let mut stable = inventory == git.read(&cwd, &command, MAX_INVENTORY_BYTES)?;
        if working_selection {
            stable &= untracked == git.read(&cwd, &untracked_command, MAX_INVENTORY_BYTES)?;
            let flags = git.read(&cwd, &["ls-files", "-v", "-z"], MAX_INVENTORY_BYTES)?;
            validate_index_flags(&flags)?;
            stable &= index_flags == flags;
        }
        for (path, captured) in &working {
            stable &= *captured == acquire.working(path, MAX_FILE_BYTES)?;
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
        let changes = attribute_renames(changes);
        let mut generations = vec![
            format!("{:?}", comparison.selection).into_bytes(),
            comparison
                .commit
                .as_deref()
                .unwrap_or("")
                .as_bytes()
                .to_vec(),
            configuration.generation().as_bytes().to_vec(),
            identity(&[&inventory, &index_flags, &untracked]).into_bytes(),
        ];
        for (path, file) in &working {
            let observed = match file {
                Some(file) => identity(&[
                    path.as_bytes(),
                    format!("{:?}", file.mode).as_bytes(),
                    &file.bytes,
                ]),
                None => identity(&[path.as_bytes(), b"absent"]),
            };
            generations.push(observed.into_bytes());
        }
        generations.extend(
            changes
                .iter()
                .flat_map(|c| c.before.iter().chain(c.after.iter()))
                .chain(context.iter())
                .map(|s| s.generation.as_bytes().to_vec()),
        );
        let generation = identity(&generations.iter().map(Vec::as_slice).collect::<Vec<_>>());
        let retained = changes
            .iter()
            .flat_map(|c| c.before.iter().chain(c.after.iter()))
            .chain(context.iter());
        let (mut retained_files, mut retained_bytes) = (0, 0);
        for source in retained {
            retained_files += 1;
            retained_bytes += source.bytes.len();
        }
        if let Some(bytes) = configuration.captured_bytes() {
            retained_files += 1;
            retained_bytes += bytes.len();
        }
        return Ok(Capture {
            comparison,
            generation,
            changes,
            context,
            configuration,
            stats: CaptureStats {
                retained_files,
                retained_bytes,
                acquired_files: acquire.files,
                acquired_bytes: acquire.bytes,
                attempts: attempt,
                git_processes: git.processes,
            },
        });
    }
    Err(CaptureError::Unstable)
}

fn attribute_renames(changes: Vec<Change>) -> Vec<Change> {
    let mut groups: BTreeMap<String, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
    for (index, change) in changes.iter().enumerate() {
        let (source, deletion) = match (&change.before, &change.after) {
            (Some(source), None) => (source, true),
            (None, Some(source)) => (source, false),
            _ => continue,
        };
        let key = identity(&[&source.bytes, format!("{:?}", source.git_mode).as_bytes()]);
        let group = groups.entry(key).or_default();
        if deletion {
            group.0.push(index);
        } else {
            group.1.push(index);
        }
    }
    let mut slots: Vec<_> = changes.into_iter().map(Some).collect();
    for (deletions, additions) in groups.into_values() {
        if deletions.len() != 1 || additions.len() != 1 {
            continue;
        }
        let (old, new) = (deletions[0], additions[0]);
        let before = slots[old]
            .as_ref()
            .and_then(|c| c.before.as_ref())
            .expect("deletion");
        let after = slots[new]
            .as_ref()
            .and_then(|c| c.after.as_ref())
            .expect("addition");
        if before.bytes != after.bytes || before.git_mode != after.git_mode {
            continue;
        }
        let before = slots[old].take().expect("deletion").before;
        let added = slots[new].take().expect("addition");
        slots[new] = Some(Change {
            path: added.path,
            before,
            after: added.after,
        });
    }
    let mut result: Vec<_> = slots.into_iter().flatten().collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

fn load_working(
    acquire: &mut Acquisition,
    working: &mut BTreeMap<String, Option<WorkingFile>>,
    path: &str,
) -> Result<Option<WorkingFile>, CaptureError> {
    acquire.admit_file()?;
    if let Some(file) = working.get(path) {
        if let Some(file) = file {
            acquire.admit_bytes(file.bytes.len())?;
        }
        return Ok(file.clone());
    }
    let file = acquire.working(path, acquire.remaining())?;
    if let Some(file) = &file {
        acquire.admit_bytes(file.bytes.len())?;
    }
    working.insert(path.to_owned(), file.clone());
    Ok(file)
}

fn validate_index_flags(bytes: &[u8]) -> Result<Vec<String>, CaptureError> {
    let mut paths = Vec::new();
    if !bytes.is_empty() && !bytes.ends_with(&[0]) {
        return Err(CaptureError::Unsupported("Truncated Git index flags"));
    }
    for entry in bytes.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        if entry.len() < 3 || entry[0] != b'H' || entry[1] != b' ' {
            return Err(CaptureError::Unsupported(
                "Working comparisons require a merged index without assume-unchanged or skip-worktree entries",
            ));
        }
        if paths.len() >= MAX_FILES {
            return Err(CaptureError::Limit("Index path count"));
        }
        let path = std::str::from_utf8(&entry[2..])
            .map_err(|_| CaptureError::Unsupported("Non-UTF-8 index path"))?;
        let normalized = config::normalize_path(Path::new(path))?;
        if path != normalized {
            return Err(CaptureError::Unsupported("Noncanonical Git path"));
        }
        paths.push(normalized);
    }
    Ok(paths)
}

fn inventory_paths(bytes: &[u8]) -> Result<Vec<String>, CaptureError> {
    if !bytes.is_empty() && !bytes.ends_with(&[0]) {
        return Err(CaptureError::Unsupported("Truncated Git path inventory"));
    }
    let mut result = Vec::new();
    for path in bytes.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        if result.len() >= MAX_FILES {
            return Err(CaptureError::Limit("Untracked path count"));
        }
        let path = std::str::from_utf8(path)
            .map_err(|_| CaptureError::Unsupported("Non-UTF-8 source path"))?;
        let normalized = config::normalize_path(Path::new(path))?;
        if path != normalized {
            return Err(CaptureError::Unsupported("Noncanonical Git path"));
        }
        result.push(normalized);
    }
    Ok(result)
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
