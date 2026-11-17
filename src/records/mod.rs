//! Durable retention of captured source bytes and their provenance in SQLite.
//!
//! A capture commits atomically: readers see every retained byte and identity of
//! a saved capture or none of it. A returned [`CaptureId`] follows a completed
//! commit; losing the acknowledgement after that point leaves the record
//! present, never absent.

use crate::{
    config::{self, Configuration},
    workspace::{Capture, GitMode, Role, Selection, SourceRange},
};
use rusqlite::{Connection, ErrorCode, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

/// Version stored in `PRAGMA user_version`; newer values are refused explicitly.
pub const SCHEMA_VERSION: u32 = 1;
/// Accepted default persistent-record budget per workspace (10 GiB).
pub const DEFAULT_BUDGET_BYTES: u64 = 10 * 1024 * 1024 * 1024;
/// Upper bound on waiting for a competing writer. Exceeding it reports busy.
pub const BUSY_TIMEOUT: Duration = Duration::from_secs(2);
/// `FULL` synchronous mode as read back from `PRAGMA synchronous`.
const SYNCHRONOUS_FULL: i64 = 2;

const SCHEMA: &str = "
CREATE TABLE captures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    selection_kind TEXT NOT NULL CHECK (selection_kind IN ('staged', 'unstaged', 'local', 'base')),
    selection_reference TEXT,
    resolved_commit TEXT,
    capture_generation TEXT NOT NULL,
    configuration_generation TEXT NOT NULL,
    configuration_kind TEXT NOT NULL CHECK (configuration_kind IN ('built_in', 'file')),
    configuration_file TEXT,
    configuration_sha256 TEXT,
    configuration_size INTEGER,
    configuration_bytes BLOB,
    integrity TEXT NOT NULL CHECK (length(integrity) = 64),
    CHECK ((selection_kind = 'base') = (selection_reference IS NOT NULL)),
    CHECK (
        (configuration_kind = 'built_in'
            AND configuration_file IS NULL
            AND configuration_sha256 IS NULL
            AND configuration_size IS NULL
            AND configuration_bytes IS NULL)
        OR
        (configuration_kind = 'file'
            AND configuration_file IS NOT NULL
            AND configuration_sha256 IS NOT NULL
            AND configuration_size IS NOT NULL
            AND configuration_bytes IS NOT NULL
            AND length(configuration_bytes) = configuration_size)
    )
);
CREATE TABLE capture_changes (
    capture_id INTEGER NOT NULL REFERENCES captures (id),
    ordinal INTEGER NOT NULL,
    path TEXT NOT NULL,
    PRIMARY KEY (capture_id, ordinal)
);
CREATE TABLE capture_sources (
    capture_id INTEGER NOT NULL REFERENCES captures (id),
    ordinal INTEGER NOT NULL,
    change_ordinal INTEGER,
    side TEXT NOT NULL CHECK (side IN ('before', 'after', 'context')),
    path TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('head', 'base', 'index', 'working', 'context')),
    generation TEXT NOT NULL,
    git_blob TEXT,
    git_mode TEXT CHECK (git_mode IS NULL OR git_mode IN ('regular', 'executable')),
    bytes BLOB NOT NULL,
    PRIMARY KEY (capture_id, ordinal),
    CHECK ((side = 'context') = (change_ordinal IS NULL))
);
";

/// Explicit persistent-record budget for one store, in bytes.
///
/// This is an application admission policy, not a filesystem quota. It is a
/// per-store input and is not stored in the database, so callers own the
/// policy that reopens a store with a budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    limit: u64,
}

impl Budget {
    pub const fn new(limit: u64) -> Self {
        Self { limit }
    }
    pub const fn limit(self) -> u64 {
        self.limit
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::new(DEFAULT_BUDGET_BYTES)
    }
}

/// Actual retained storage plus the warning and over-budget state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageUsage {
    pub retained: u64,
    pub limit: u64,
    pub warning: bool,
    pub over_limit: bool,
}

/// Assigned only after the capture transaction commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptureId(pub i64);

/// A committed capture and the actual footprint observed after its commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedCapture {
    pub id: CaptureId,
    pub usage: StorageUsage,
}

impl std::fmt::Display for CaptureId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Capture store is unavailable: {0}")]
    Unavailable(String),
    #[error("Capture store schema version {found} is unsupported; this build supports {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("Database is not an Anti-Slop capture store")]
    NotACaptureStore,
    #[error("Another writer holds the capture store beyond the {BUSY_TIMEOUT:?} bound")]
    Busy,
    #[error("Capture store is damaged: {0}")]
    Corrupt(String),
    #[error("Capture store operation failed: {0}")]
    Failed(String),
    #[error("Capture record {0} does not exist")]
    NotFound(CaptureId),
    #[error("Byte range does not bind a retained source")]
    UnboundRange,
    #[error(
        "Collection budget of {limit} bytes would be exceeded: {retained} bytes retained, admission needs {required}"
    )]
    BudgetExceeded {
        retained: u64,
        limit: u64,
        required: u64,
    },
    #[error("Capture {id} was committed, but its retained storage could not be measured: {reason}")]
    CommittedUnmeasured { id: CaptureId, reason: String },
}

pub struct Store {
    connection: Connection,
    path: PathBuf,
    budget: Budget,
    page_size: u64,
}

impl Store {
    /// Opens the store at `path` with the accepted 10 GiB default budget.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::open_with_budget(path, Budget::default())
    }

    /// Opens the store with an explicit admission budget.
    ///
    /// Opening does not reject a store whose retained data already exceeds the
    /// budget: retained reads remain available and only new collection is
    /// refused. Initialization is not denied for a low budget, but the schema
    /// and file bytes it creates are included in every later admission
    /// measurement.
    pub fn open_with_budget(path: &Path, budget: Budget) -> Result<Self, StoreError> {
        let mut connection = Connection::open(path).map_err(database_error)?;
        // SQLite resolves symlinks and normalizes the path before deriving its
        // auxiliary filenames. Canonicalizing the now-existing database once
        // uses that same effective identity instead of maintaining a second
        // symlink resolver.
        let path = fs::canonicalize(path).map_err(|error| {
            StoreError::Unavailable(format!("Cannot resolve the capture store path: {error}"))
        })?;
        connection
            .busy_timeout(BUSY_TIMEOUT)
            .map_err(database_error)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(database_error)?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(database_error)?;
        let version = u32::try_from(version)
            .map_err(|_| StoreError::Corrupt("Stored schema version is negative".into()))?;
        match version {
            0 => {
                configure(&connection)?;
                initialize(&mut connection)?;
            }
            SCHEMA_VERSION => {
                verify(&connection)?;
                configure(&connection)?;
            }
            found => {
                return Err(StoreError::UnsupportedSchema {
                    found,
                    supported: SCHEMA_VERSION,
                });
            }
        }
        let page_size: i64 = connection
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .map_err(database_error)?;
        let page_size = u64::try_from(page_size)
            .map_err(|_| StoreError::Corrupt("Stored page size is negative".into()))?;
        if page_size < 480 {
            return Err(StoreError::Failed(
                "SQLite page size is below the supported minimum".into(),
            ));
        }
        Ok(Self {
            connection,
            path,
            budget,
            page_size,
        })
    }

    /// Reports retained storage and whether it crossed the 80% warning line.
    ///
    /// The value is a snapshot of the main database plus its existing `-wal`,
    /// `-shm` and `-journal` files. SQLite removes `-wal`/`-shm` when the last
    /// connection closes, so a later call can legitimately report less.
    pub fn usage(&self) -> Result<StorageUsage, StoreError> {
        let retained = storage_footprint(&self.path)?.total;
        let limit = self.budget.limit();
        Ok(StorageUsage {
            retained,
            limit,
            warning: retained > 0 && u128::from(retained) * 5 >= u128::from(limit) * 4,
            over_limit: retained > limit,
        })
    }

    /// Retains every byte and identity of `capture` in one committed transaction.
    ///
    /// The returned usage is measured after the commit. A committed record whose
    /// measured footprint exceeds the budget is not hidden: the result reports
    /// `over_limit`, and the record remains committed and readable.
    pub fn save_capture(&mut self, capture: &Capture) -> Result<SavedCapture, StoreError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(database_error)?;
        let (selection_kind, selection_reference) = selection_columns(capture.selection());
        let configuration = capture.configuration();
        let columns = configuration_columns(configuration)?;
        let changes = capture
            .changes()
            .iter()
            .enumerate()
            .map(|(ordinal, change)| ChangeView {
                ordinal: ordinal as i64,
                path: &change.path,
            })
            .collect::<Vec<_>>();
        let mut sources = Vec::new();
        let mut ordinal = 0_i64;
        for (change_ordinal, change) in capture.changes().iter().enumerate() {
            for (side, source) in [
                ("before", change.before.as_ref()),
                ("after", change.after.as_ref()),
            ] {
                if let Some(source) = source {
                    sources.push(SourceView {
                        ordinal,
                        change_ordinal: Some(change_ordinal as i64),
                        side,
                        path: &source.path,
                        role: role_code(source.role),
                        generation: &source.generation,
                        git_blob: source.git_blob.as_deref(),
                        git_mode: source.git_mode.map(mode_code),
                        bytes: source.bytes(),
                    });
                    ordinal += 1;
                }
            }
        }
        for source in capture.context() {
            sources.push(SourceView {
                ordinal,
                change_ordinal: None,
                side: "context",
                path: &source.path,
                role: role_code(source.role),
                generation: &source.generation,
                git_blob: source.git_blob.as_deref(),
                git_mode: source.git_mode.map(mode_code),
                bytes: source.bytes(),
            });
            ordinal += 1;
        }
        // Admission happens after BEGIN IMMEDIATE, so competing writers are
        // serialized through commit: the next writer measures the committed
        // result instead of the capacity this writer reserved. No separate
        // reservation ledger exists to leak after a crash.
        let footprint = storage_footprint(&self.path)?;
        let retained = footprint.total;
        let header_metadata = selection_reference.map_or(0, |value| value.len() as u64)
            + capture
                .resolved_commit()
                .map_or(0, |value| value.len() as u64)
            + columns.file.map_or(0, |value| value.len() as u64)
            + columns.sha256.map_or(0, |value| value.len() as u64)
            + capture.generation().len() as u64
            + configuration.generation().len() as u64;
        let reservation = reservation_bytes(
            self.page_size,
            &changes,
            &sources,
            columns.bytes,
            footprint.wal,
            header_metadata,
        )?;
        let required = retained.checked_add(reservation).ok_or_else(|| {
            StoreError::Failed("Capture storage admission exceeds the supported byte range".into())
        })?;
        if required > self.budget.limit() {
            return Err(StoreError::BudgetExceeded {
                retained,
                limit: self.budget.limit(),
                required,
            });
        }
        let integrity = record_integrity(
            &HeaderView {
                selection_kind,
                selection_reference,
                resolved_commit: capture.resolved_commit(),
                generation: capture.generation(),
                configuration_generation: configuration.generation(),
                configuration_kind: columns.kind,
                configuration_file: columns.file,
                configuration_sha256: columns.sha256,
                configuration_size: columns.size,
                configuration_bytes: columns.bytes,
            },
            &changes,
            &sources,
        );
        transaction
            .execute(
                "INSERT INTO captures (
                    selection_kind, selection_reference, resolved_commit, capture_generation,
                    configuration_generation, configuration_kind, configuration_file,
                    configuration_sha256, configuration_size, configuration_bytes, integrity
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    selection_kind,
                    selection_reference,
                    capture.resolved_commit(),
                    capture.generation(),
                    configuration.generation(),
                    columns.kind,
                    columns.file,
                    columns.sha256,
                    columns.size,
                    columns.bytes,
                    integrity,
                ],
            )
            .map_err(database_error)?;
        let id = transaction.last_insert_rowid();
        for change in &changes {
            transaction
                .execute(
                    "INSERT INTO capture_changes (capture_id, ordinal, path) VALUES (?1, ?2, ?3)",
                    params![id, change.ordinal, change.path],
                )
                .map_err(database_error)?;
        }
        for source in &sources {
            transaction
                .execute(
                    "INSERT INTO capture_sources (
                        capture_id, ordinal, change_ordinal, side, path, role, generation,
                        git_blob, git_mode, bytes
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        id,
                        source.ordinal,
                        source.change_ordinal,
                        source.side,
                        source.path,
                        source.role,
                        source.generation,
                        source.git_blob,
                        source.git_mode,
                        source.bytes,
                    ],
                )
                .map_err(database_error)?;
        }
        transaction.commit().map_err(database_error)?;
        let id = CaptureId(id);
        let usage = self
            .usage()
            .map_err(|error| StoreError::CommittedUnmeasured {
                id,
                reason: error.to_string(),
            })?;
        Ok(SavedCapture { id, usage })
    }

    /// Reads a committed capture and all of its retained bytes in one snapshot.
    ///
    /// The retained membership, provenance and bytes are verified against the
    /// integrity digest written with the capture before anything is returned.
    pub fn read_capture(&self, id: CaptureId) -> Result<RetainedCapture, StoreError> {
        let transaction = self
            .connection
            .unchecked_transaction()
            .map_err(database_error)?;
        let stored = transaction
            .query_row(
                "SELECT selection_kind, selection_reference, resolved_commit, capture_generation,
                        configuration_generation, configuration_kind, configuration_file,
                        configuration_sha256, configuration_size, configuration_bytes, integrity
                 FROM captures WHERE id = ?1",
                [id.0],
                |row| {
                    Ok(StoredCapture {
                        selection_kind: row.get(0)?,
                        selection_reference: row.get(1)?,
                        resolved_commit: row.get(2)?,
                        generation: row.get(3)?,
                        configuration_generation: row.get(4)?,
                        configuration_kind: row.get(5)?,
                        configuration_file: row.get(6)?,
                        configuration_sha256: row.get(7)?,
                        configuration_size: row.get(8)?,
                        configuration_bytes: row.get(9)?,
                        integrity: row.get(10)?,
                    })
                },
            )
            .optional()
            .map_err(database_error)?
            .ok_or(StoreError::NotFound(id))?;

        let mut stored_changes = Vec::new();
        {
            let mut statement = transaction
                .prepare(
                    "SELECT ordinal, path FROM capture_changes
                     WHERE capture_id = ?1 ORDER BY ordinal",
                )
                .map_err(database_error)?;
            let rows = statement
                .query_map([id.0], |row| {
                    Ok(StoredChange {
                        ordinal: row.get(0)?,
                        path: row.get(1)?,
                    })
                })
                .map_err(database_error)?;
            for row in rows {
                stored_changes.push(row.map_err(database_error)?);
            }
        }

        let mut stored_sources = Vec::new();
        {
            let mut statement = transaction
                .prepare(
                    "SELECT ordinal, change_ordinal, side, path, role, generation,
                            git_blob, git_mode, bytes
                     FROM capture_sources WHERE capture_id = ?1 ORDER BY ordinal",
                )
                .map_err(database_error)?;
            let rows = statement
                .query_map([id.0], |row| {
                    Ok(StoredSource {
                        ordinal: row.get(0)?,
                        change_ordinal: row.get(1)?,
                        side: row.get(2)?,
                        path: row.get(3)?,
                        role: row.get(4)?,
                        generation: row.get(5)?,
                        git_blob: row.get(6)?,
                        git_mode: row.get(7)?,
                        bytes: row.get(8)?,
                    })
                })
                .map_err(database_error)?;
            for row in rows {
                stored_sources.push(row.map_err(database_error)?);
            }
        }

        let change_views = stored_changes
            .iter()
            .map(|change| ChangeView {
                ordinal: change.ordinal,
                path: &change.path,
            })
            .collect::<Vec<_>>();
        let source_views = stored_sources
            .iter()
            .map(|source| SourceView {
                ordinal: source.ordinal,
                change_ordinal: source.change_ordinal,
                side: &source.side,
                path: &source.path,
                role: &source.role,
                generation: &source.generation,
                git_blob: source.git_blob.as_deref(),
                git_mode: source.git_mode.as_deref(),
                bytes: &source.bytes,
            })
            .collect::<Vec<_>>();
        let computed = record_integrity(
            &HeaderView {
                selection_kind: &stored.selection_kind,
                selection_reference: stored.selection_reference.as_deref(),
                resolved_commit: stored.resolved_commit.as_deref(),
                generation: &stored.generation,
                configuration_generation: &stored.configuration_generation,
                configuration_kind: &stored.configuration_kind,
                configuration_file: stored.configuration_file.as_deref(),
                configuration_sha256: stored.configuration_sha256.as_deref(),
                configuration_size: stored.configuration_size,
                configuration_bytes: stored.configuration_bytes.as_deref(),
            },
            &change_views,
            &source_views,
        );
        if computed != stored.integrity {
            return Err(StoreError::Corrupt(
                "Retained capture failed its integrity check".into(),
            ));
        }

        let mut changes = Vec::new();
        let mut change_index = BTreeMap::new();
        for change in stored_changes {
            change_index.insert(change.ordinal, changes.len());
            changes.push(RetainedChange {
                path: change.path,
                before: None,
                after: None,
            });
        }
        let mut context = Vec::new();
        for source in stored_sources {
            let StoredSource {
                change_ordinal,
                side,
                path,
                role,
                generation,
                git_blob,
                git_mode,
                bytes,
                ..
            } = source;
            let source = RetainedSource {
                path,
                role: role_from_code(&role)?,
                generation,
                git_blob,
                git_mode: git_mode.as_deref().map(mode_from_code).transpose()?,
                bytes,
            };
            match (change_ordinal, side.as_str()) {
                (Some(ordinal), "before" | "after") => {
                    let index = *change_index.get(&ordinal).ok_or_else(|| {
                        StoreError::Corrupt(format!(
                            "Source refers to unknown change ordinal {ordinal}"
                        ))
                    })?;
                    let slot = if side == "before" {
                        &mut changes[index].before
                    } else {
                        &mut changes[index].after
                    };
                    if slot.is_some() {
                        return Err(StoreError::Corrupt(format!(
                            "Change {ordinal} retains more than one {side} source"
                        )));
                    }
                    *slot = Some(source);
                }
                (None, "context") => context.push(source),
                _ => {
                    return Err(StoreError::Corrupt(
                        "Stored source has inconsistent side and change ordinal".into(),
                    ));
                }
            }
        }

        Ok(RetainedCapture {
            id,
            selection: selection_from_columns(&stored.selection_kind, stored.selection_reference)?,
            resolved_commit: stored.resolved_commit,
            generation: stored.generation,
            configuration: configuration_from_columns(
                stored.configuration_generation,
                &stored.configuration_kind,
                stored.configuration_file,
                stored.configuration_sha256,
                stored.configuration_size,
                stored.configuration_bytes,
            )?,
            changes,
            context,
        })
    }
}

fn configure(connection: &Connection) -> Result<(), StoreError> {
    let mode: String = connection
        .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
        .map_err(database_error)?;
    if mode != "wal" {
        return Err(StoreError::Failed(format!(
            "WAL journal mode is unavailable; SQLite reported {mode}"
        )));
    }
    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(database_error)?;
    let synchronous: i64 = connection
        .query_row("PRAGMA synchronous", [], |row| row.get(0))
        .map_err(database_error)?;
    if synchronous != SYNCHRONOUS_FULL {
        return Err(StoreError::Failed(format!(
            "FULL synchronous mode is unavailable; SQLite reported {synchronous}"
        )));
    }
    Ok(())
}

fn initialize(connection: &mut Connection) -> Result<(), StoreError> {
    let existing: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if existing != 0 {
        return Err(StoreError::NotACaptureStore);
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(database_error)?;
    transaction.execute_batch(SCHEMA).map_err(database_error)?;
    transaction
        .pragma_update(None, "user_version", i64::from(SCHEMA_VERSION))
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)
}

fn verify(connection: &Connection) -> Result<(), StoreError> {
    let present: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table'
             AND name IN ('captures', 'capture_changes', 'capture_sources')",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    let integrity: i64 = connection
        .query_row(
            "SELECT count(*) FROM pragma_table_info('captures') WHERE name = 'integrity'",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if present != 3 || integrity != 1 {
        return Err(StoreError::NotACaptureStore);
    }
    Ok(())
}

struct StoredCapture {
    selection_kind: String,
    selection_reference: Option<String>,
    resolved_commit: Option<String>,
    generation: String,
    configuration_generation: String,
    configuration_kind: String,
    configuration_file: Option<String>,
    configuration_sha256: Option<String>,
    configuration_size: Option<i64>,
    configuration_bytes: Option<Vec<u8>>,
    integrity: String,
}

struct StoredChange {
    ordinal: i64,
    path: String,
}

struct StoredSource {
    ordinal: i64,
    change_ordinal: Option<i64>,
    side: String,
    path: String,
    role: String,
    generation: String,
    git_blob: Option<String>,
    git_mode: Option<String>,
    bytes: Vec<u8>,
}

/// Retained source bytes with the exact path, role and generation they bind.
pub struct RetainedSource {
    pub path: String,
    pub role: Role,
    pub generation: String,
    pub git_blob: Option<String>,
    pub git_mode: Option<GitMode>,
    bytes: Vec<u8>,
}

impl RetainedSource {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::fmt::Debug for RetainedSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RetainedSource")
            .field("path", &self.path)
            .field("role", &self.role)
            .field("generation", &self.generation)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

#[derive(Debug)]
pub struct RetainedChange {
    pub path: String,
    pub before: Option<RetainedSource>,
    pub after: Option<RetainedSource>,
}

pub struct RetainedConfiguration {
    generation: String,
    source: config::Source,
    captured: Option<Vec<u8>>,
}

impl RetainedConfiguration {
    pub fn generation(&self) -> &str {
        &self.generation
    }
    pub fn source(&self) -> &config::Source {
        &self.source
    }
    pub fn captured_bytes(&self) -> Option<&[u8]> {
        self.captured.as_deref()
    }
}

impl std::fmt::Debug for RetainedConfiguration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RetainedConfiguration")
            .field("generation", &self.generation)
            .field("source", &self.source)
            .finish()
    }
}

pub struct RetainedCapture {
    id: CaptureId,
    selection: Selection,
    resolved_commit: Option<String>,
    generation: String,
    configuration: RetainedConfiguration,
    changes: Vec<RetainedChange>,
    context: Vec<RetainedSource>,
}

impl RetainedCapture {
    pub fn id(&self) -> CaptureId {
        self.id
    }
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    pub fn resolved_commit(&self) -> Option<&str> {
        self.resolved_commit.as_deref()
    }
    pub fn generation(&self) -> &str {
        &self.generation
    }
    pub fn configuration(&self) -> &RetainedConfiguration {
        &self.configuration
    }
    pub fn changes(&self) -> &[RetainedChange] {
        &self.changes
    }
    pub fn context(&self) -> &[RetainedSource] {
        &self.context
    }
    /// Resolves a byte range only against the retained path, role and generation.
    pub fn slice(&self, range: &SourceRange) -> Result<&[u8], StoreError> {
        self.changes
            .iter()
            .flat_map(|change| change.before.iter().chain(change.after.iter()))
            .chain(self.context.iter())
            .find(|source| {
                source.path == range.path
                    && source.role == range.role
                    && source.generation == range.generation
            })
            .and_then(|source| source.bytes.get(range.range.clone()))
            .ok_or(StoreError::UnboundRange)
    }
}

struct HeaderView<'a> {
    selection_kind: &'a str,
    selection_reference: Option<&'a str>,
    resolved_commit: Option<&'a str>,
    generation: &'a str,
    configuration_generation: &'a str,
    configuration_kind: &'a str,
    configuration_file: Option<&'a str>,
    configuration_sha256: Option<&'a str>,
    configuration_size: Option<i64>,
    configuration_bytes: Option<&'a [u8]>,
}

struct ChangeView<'a> {
    ordinal: i64,
    path: &'a str,
}

struct SourceView<'a> {
    ordinal: i64,
    change_ordinal: Option<i64>,
    side: &'a str,
    path: &'a str,
    role: &'a str,
    generation: &'a str,
    git_blob: Option<&'a str>,
    git_mode: Option<&'a str>,
    bytes: &'a [u8],
}

const INTEGRITY_TAG: &[u8] = b"anti-slop.records.integrity.v1";

struct Integrity(Sha256);

impl Integrity {
    fn new() -> Self {
        let mut digest = Sha256::new();
        digest.update((INTEGRITY_TAG.len() as u64).to_le_bytes());
        digest.update(INTEGRITY_TAG);
        Self(digest)
    }
    fn integer(&mut self, value: i64) {
        self.0.update(value.to_le_bytes());
    }
    fn text(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }
    fn optional_text(&mut self, value: Option<&str>) {
        match value {
            Some(value) => {
                self.integer(1);
                self.text(value);
            }
            None => self.integer(0),
        }
    }
    fn optional_integer(&mut self, value: Option<i64>) {
        match value {
            Some(value) => {
                self.integer(1);
                self.integer(value);
            }
            None => self.integer(0),
        }
    }
    fn bytes(&mut self, value: &[u8]) {
        self.integer(value.len() as i64);
        self.0.update(value);
    }
    fn optional_bytes(&mut self, value: Option<&[u8]>) {
        match value {
            Some(value) => {
                self.integer(1);
                self.bytes(value);
            }
            None => self.integer(0),
        }
    }
    fn finish(self) -> String {
        self.0
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

/// Domain-separated digest over the complete retained membership, provenance
/// and bytes. It is written once with the capture and recomputed on every read.
fn record_integrity(
    header: &HeaderView<'_>,
    changes: &[ChangeView<'_>],
    sources: &[SourceView<'_>],
) -> String {
    let mut integrity = Integrity::new();
    integrity.text(header.selection_kind);
    integrity.optional_text(header.selection_reference);
    integrity.optional_text(header.resolved_commit);
    integrity.text(header.generation);
    integrity.text(header.configuration_generation);
    integrity.text(header.configuration_kind);
    integrity.optional_text(header.configuration_file);
    integrity.optional_text(header.configuration_sha256);
    integrity.optional_integer(header.configuration_size);
    integrity.optional_bytes(header.configuration_bytes);
    integrity.integer(changes.len() as i64);
    for change in changes {
        integrity.integer(change.ordinal);
        integrity.text(change.path);
    }
    integrity.integer(sources.len() as i64);
    for source in sources {
        integrity.integer(source.ordinal);
        integrity.optional_integer(source.change_ordinal);
        integrity.text(source.side);
        integrity.text(source.path);
        integrity.text(source.role);
        integrity.text(source.generation);
        integrity.optional_text(source.git_blob);
        integrity.optional_text(source.git_mode);
        integrity.bytes(source.bytes);
    }
    integrity.finish()
}

fn selection_columns(selection: &Selection) -> (&'static str, Option<&str>) {
    match selection {
        Selection::Staged => ("staged", None),
        Selection::Unstaged => ("unstaged", None),
        Selection::Local => ("local", None),
        Selection::Base(reference) => ("base", Some(reference)),
    }
}

fn selection_from_columns(kind: &str, reference: Option<String>) -> Result<Selection, StoreError> {
    match (kind, reference) {
        ("staged", None) => Ok(Selection::Staged),
        ("unstaged", None) => Ok(Selection::Unstaged),
        ("local", None) => Ok(Selection::Local),
        ("base", Some(reference)) => Ok(Selection::Base(reference)),
        _ => Err(StoreError::Corrupt(
            "Stored comparison selection is inconsistent".into(),
        )),
    }
}

struct ConfigurationColumns<'a> {
    kind: &'static str,
    file: Option<&'a str>,
    sha256: Option<&'a str>,
    size: Option<i64>,
    bytes: Option<&'a [u8]>,
}

fn configuration_columns(
    configuration: &Configuration,
) -> Result<ConfigurationColumns<'_>, StoreError> {
    match configuration.source() {
        config::Source::BuiltIn => Ok(ConfigurationColumns {
            kind: "built_in",
            file: None,
            sha256: None,
            size: None,
            bytes: None,
        }),
        config::Source::File {
            file,
            sha256,
            bytes,
        } => {
            let file = file.to_str().ok_or_else(|| {
                StoreError::Failed("Configuration source path is not UTF-8".into())
            })?;
            let captured = configuration.captured_bytes().ok_or_else(|| {
                StoreError::Failed("Captured configuration bytes are unavailable".into())
            })?;
            Ok(ConfigurationColumns {
                kind: "file",
                file: Some(file),
                sha256: Some(sha256),
                size: Some(*bytes as i64),
                bytes: Some(captured),
            })
        }
    }
}

fn configuration_from_columns(
    generation: String,
    kind: &str,
    file: Option<String>,
    sha256: Option<String>,
    size: Option<i64>,
    bytes: Option<Vec<u8>>,
) -> Result<RetainedConfiguration, StoreError> {
    let source = match kind {
        "built_in" => {
            if file.is_some() || sha256.is_some() || size.is_some() || bytes.is_some() {
                return Err(StoreError::Corrupt(
                    "Stored built-in configuration carries file provenance".into(),
                ));
            }
            config::Source::BuiltIn
        }
        "file" => {
            let (file, sha256, size, bytes) = match (file, sha256, size, bytes) {
                (Some(file), Some(sha256), Some(size), Some(bytes)) => (file, sha256, size, bytes),
                _ => {
                    return Err(StoreError::Corrupt(
                        "Stored configuration is missing file provenance".into(),
                    ));
                }
            };
            let size = usize::try_from(size)
                .map_err(|_| StoreError::Corrupt("Stored configuration size is negative".into()))?;
            if size != bytes.len() {
                return Err(StoreError::Corrupt(
                    "Stored configuration size does not match its bytes".into(),
                ));
            }
            return Ok(RetainedConfiguration {
                generation,
                source: config::Source::File {
                    file: file.into(),
                    sha256,
                    bytes: size,
                },
                captured: Some(bytes),
            });
        }
        _ => {
            return Err(StoreError::Corrupt(
                "Stored configuration kind is unknown".into(),
            ));
        }
    };
    Ok(RetainedConfiguration {
        generation,
        source,
        captured: None,
    })
}

fn role_code(role: Role) -> &'static str {
    match role {
        Role::Head => "head",
        Role::Base => "base",
        Role::Index => "index",
        Role::Working => "working",
        Role::Context => "context",
        Role::Configuration => "configuration",
    }
}

fn role_from_code(code: &str) -> Result<Role, StoreError> {
    match code {
        "head" => Ok(Role::Head),
        "base" => Ok(Role::Base),
        "index" => Ok(Role::Index),
        "working" => Ok(Role::Working),
        "context" => Ok(Role::Context),
        _ => Err(StoreError::Corrupt(format!(
            "Stored source role {code} is unknown"
        ))),
    }
}

fn mode_code(mode: GitMode) -> &'static str {
    match mode {
        GitMode::Regular => "regular",
        GitMode::Executable => "executable",
    }
}

fn mode_from_code(code: &str) -> Result<GitMode, StoreError> {
    match code {
        "regular" => Ok(GitMode::Regular),
        "executable" => Ok(GitMode::Executable),
        other => Err(StoreError::Corrupt(format!(
            "Stored Git mode {other} is unknown"
        ))),
    }
}

const BASE_METADATA_BYTES: u64 = 4096;
const CHANGE_METADATA_BYTES: u64 = 256;
const SOURCE_METADATA_BYTES: u64 = 512;
const WAL_FRAME_OVERHEAD: u64 = 24;
const WAL_HEADER_BYTES: u64 = 32;
const AUXILIARY_HEADROOM_BYTES: u64 = 64 * 1024;

/// Upper bound on the transient and retained growth of one admitted capture.
///
/// Assumptions, all conservative for this store:
///
/// - WAL mode with `synchronous = FULL` (set by `configure`); frames have a
///   24-byte header after a 32-byte WAL header, and auto-checkpoints run at
///   about 1000 pages in the committing thread.
/// - The database uses its measured page size with zero reserved bytes, as in
///   the bundled default build; an overflow page carries at most `page_size - 4`
///   payload bytes.
/// - The workspace capture limits bound the inputs: at most 512 source/context
///   observations, 1 MiB per input, 16 MiB of admitted content and a 1 MiB
///   configuration; paths and the base reference are at most 4,096 bytes, and
///   generation/blob identities are short fixed strings. `BASE_METADATA_BYTES`
///   covers fixed capture-row and page/cell metadata, `CHANGE_METADATA_BYTES`
///   covers a change row beyond its path, and `SOURCE_METADATA_BYTES` covers a
///   source row beyond its dynamic path/generation (role, mode, side, Git blob
///   and record/cell headers). Dynamic path, generation, configuration-path,
///   reference and hash lengths are added explicitly.
/// - All payload is charged as if it needed overflow pages; every change,
///   source and the capture row also gets eight b-tree pages. For the bounded
///   row counts this is a conservative engineering allowance for leaf, interior
///   and split pages, not a formal worst-case proof for arbitrary storage. The
///   resulting WAL
///   is charged twice for the transaction and a checkpoint that can leave the
///   same content in both WAL and database; the existing WAL is added again
///   because a checkpoint may backfill already committed frames into the
///   database while the file remains; 64 KiB covers `-shm`, WAL headers and
///   rounding.
///
/// The constants are conservative engineering allowances validated against real
/// near-boundary peaks, not a formal storage bound for arbitrary filesystems.
/// This is an application admission bound, not a filesystem quota and not a
/// hard storage guarantee. `journal_size_limit` is not used as a peak bound.
fn reservation_bytes(
    page_size: u64,
    changes: &[ChangeView<'_>],
    sources: &[SourceView<'_>],
    configuration_bytes: Option<&[u8]>,
    checkpoint_backlog: u64,
    header_metadata: u64,
) -> Result<u64, StoreError> {
    fn overflow() -> StoreError {
        StoreError::Failed("Capture storage admission overflows the supported byte range".into())
    }
    let mut payload = BASE_METADATA_BYTES
        .checked_add(header_metadata)
        .ok_or_else(overflow)?;
    for change in changes {
        payload = payload
            .checked_add(CHANGE_METADATA_BYTES)
            .and_then(|value| value.checked_add(change.path.len() as u64))
            .ok_or_else(overflow)?;
    }
    for source in sources {
        payload = payload
            .checked_add(SOURCE_METADATA_BYTES)
            .and_then(|value| value.checked_add(source.path.len() as u64))
            .and_then(|value| value.checked_add(source.generation.len() as u64))
            .and_then(|value| value.checked_add(source.bytes.len() as u64))
            .ok_or_else(overflow)?;
    }
    if let Some(bytes) = configuration_bytes {
        payload = payload
            .checked_add(bytes.len() as u64)
            .ok_or_else(overflow)?;
    }
    let usable = page_size.saturating_sub(4).max(1);
    let row_pages = (changes.len() as u64)
        .checked_add(sources.len() as u64)
        .and_then(|rows| rows.checked_add(1))
        .and_then(|rows| rows.checked_mul(8))
        .ok_or_else(overflow)?;
    let pages = payload
        .div_ceil(usable)
        .checked_add(row_pages)
        .and_then(|value| value.checked_add(16))
        .ok_or_else(overflow)?;
    pages
        .checked_mul(page_size.saturating_add(WAL_FRAME_OVERHEAD))
        .and_then(|value| value.checked_add(WAL_HEADER_BYTES))
        .and_then(|value| value.checked_mul(2))
        .and_then(|value| value.checked_add(checkpoint_backlog))
        .and_then(|value| value.checked_add(AUXILIARY_HEADROOM_BYTES))
        .ok_or_else(overflow)
}

fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

struct StorageFootprint {
    total: u64,
    wal: u64,
}

/// Actual bytes occupied by the database and its SQLite auxiliary files.
fn storage_footprint(path: &Path) -> Result<StorageFootprint, StoreError> {
    let database = fs::metadata(path).map_err(|error| {
        StoreError::Unavailable(format!("Cannot measure the capture store: {error}"))
    })?;
    if !database.is_file() {
        return Err(StoreError::Unavailable(
            "The capture store path is not a regular file".into(),
        ));
    }
    let mut total = database.len();
    let mut wal = 0;
    for suffix in ["-wal", "-shm", "-journal"] {
        let candidate = suffixed(path, suffix);
        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() => {
                if suffix == "-wal" {
                    wal = metadata.len();
                }
                total = total.checked_add(metadata.len()).ok_or_else(|| {
                    StoreError::Unavailable("Capture store size exceeds the supported range".into())
                })?;
            }
            Ok(_) => {
                return Err(StoreError::Unavailable(format!(
                    "Capture store auxiliary path {} is not a regular file",
                    candidate.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(StoreError::Unavailable(format!(
                    "Cannot measure capture store auxiliary storage: {error}"
                )));
            }
        }
    }
    Ok(StorageFootprint { total, wal })
}

fn database_error(error: rusqlite::Error) -> StoreError {
    match error {
        rusqlite::Error::SqliteFailure(failure, message) => {
            let detail = message.unwrap_or_else(|| "SQLite reported a failure".into());
            match failure.code {
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked => StoreError::Busy,
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase => StoreError::Corrupt(detail),
                ErrorCode::CannotOpen
                | ErrorCode::ReadOnly
                | ErrorCode::SystemIoFailure
                | ErrorCode::PermissionDenied => StoreError::Unavailable(detail),
                _ => StoreError::Failed(detail),
            }
        }
        other => StoreError::Failed(other.to_string()),
    }
}
