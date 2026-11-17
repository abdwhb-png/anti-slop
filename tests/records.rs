//! Public-boundary qualification of durable capture recording and retained reads.

#[path = "support/records/mod.rs"]
mod support;

use anti_slop::{
    config::Source as ConfigurationSource,
    records::{
        BUSY_TIMEOUT, Budget, CaptureId, DEFAULT_BUDGET_BYTES, RetainedCapture, Store, StoreError,
    },
    workspace::{Capture, CaptureRequest, GitMode, Role, Selection, Source, SourceRange, capture},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Head => "head",
        Role::Base => "base",
        Role::Index => "index",
        Role::Working => "working",
        Role::Context => "context",
        Role::Configuration => "configuration",
    }
}

fn role_from_name(name: &str) -> Role {
    match name {
        "head" => Role::Head,
        "base" => Role::Base,
        "index" => Role::Index,
        "working" => Role::Working,
        "context" => Role::Context,
        "configuration" => Role::Configuration,
        other => panic!("unknown role {other}"),
    }
}

fn mode_name(mode: GitMode) -> &'static str {
    match mode {
        GitMode::Regular => "regular",
        GitMode::Executable => "executable",
    }
}

fn selection_name(selection: &Selection) -> String {
    match selection {
        Selection::Staged => "staged".into(),
        Selection::Unstaged => "unstaged".into(),
        Selection::Local => "local".into(),
        Selection::Base(reference) => format!("base:{reference}"),
    }
}

fn selection_from_name(name: &str) -> Selection {
    match name {
        "staged" => Selection::Staged,
        "unstaged" => Selection::Unstaged,
        "local" => Selection::Local,
        _ => Selection::Base(name.trim_start_matches("base:").to_owned()),
    }
}

fn configuration_source_json(source: &ConfigurationSource) -> Value {
    match source {
        ConfigurationSource::BuiltIn => json!({"kind": "built_in"}),
        ConfigurationSource::File {
            file,
            sha256,
            bytes,
        } => json!({
            "kind": "file",
            "file": file.to_string_lossy(),
            "sha256": sha256,
            "bytes": bytes,
        }),
    }
}

fn captured_source_json(source: &Source) -> Value {
    json!({
        "path": &source.path,
        "role": role_name(source.role),
        "generation": &source.generation,
        "git_blob": &source.git_blob,
        "git_mode": source.git_mode.map(mode_name),
        "bytes": support::hex(source.bytes()),
    })
}

fn retained_source_json(source: &anti_slop::records::RetainedSource) -> Value {
    json!({
        "path": &source.path,
        "role": role_name(source.role),
        "generation": &source.generation,
        "git_blob": &source.git_blob,
        "git_mode": source.git_mode.map(mode_name),
        "bytes": support::hex(source.bytes()),
    })
}

fn change_json(before: &Option<Source>, after: &Option<Source>) -> Value {
    json!({
        "before": before.as_ref().map(captured_source_json),
        "after": after.as_ref().map(captured_source_json),
    })
}

fn captured_json(capture: &Capture, slice: Option<String>) -> Value {
    let configuration = capture.configuration();
    json!({
        "id": null,
        "selection": selection_name(capture.selection()),
        "resolved_commit": capture.resolved_commit(),
        "generation": capture.generation(),
        "configuration": {
            "generation": configuration.generation(),
            "source": configuration_source_json(configuration.source()),
            "captured": configuration.captured_bytes().map(support::hex),
        },
        "changes": capture
            .changes()
            .iter()
            .map(|change| {
                let mut value = change_json(&change.before, &change.after);
                value["path"] = json!(&change.path);
                value
            })
            .collect::<Vec<_>>(),
        "context": capture.context().iter().map(captured_source_json).collect::<Vec<_>>(),
        "slice": slice,
    })
}

fn retained_json(retained: &RetainedCapture, slice: Option<String>) -> Value {
    let configuration = retained.configuration();
    json!({
        "id": retained.id().0,
        "selection": selection_name(retained.selection()),
        "resolved_commit": retained.resolved_commit(),
        "generation": retained.generation(),
        "configuration": {
            "generation": configuration.generation(),
            "source": configuration_source_json(configuration.source()),
            "captured": configuration.captured_bytes().map(support::hex),
        },
        "changes": retained
            .changes()
            .iter()
            .map(|change| {
                json!({
                    "path": &change.path,
                    "before": change.before.as_ref().map(retained_source_json),
                    "after": change.after.as_ref().map(retained_source_json),
                })
            })
            .collect::<Vec<_>>(),
        "context": retained.context().iter().map(retained_source_json).collect::<Vec<_>>(),
        "slice": slice,
    })
}

fn raw(path: &Path) -> rusqlite::Connection {
    let connection = rusqlite::Connection::open(path).unwrap();
    connection.busy_timeout(Duration::from_secs(5)).unwrap();
    connection
}

fn raw_counts(path: &Path) -> (i64, i64, i64) {
    let connection = raw(path);
    let count = |table: &str| {
        connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap()
    };
    (
        count("captures"),
        count("capture_changes"),
        count("capture_sources"),
    )
}

fn raw_schema(path: &Path) -> (i64, i64, i64) {
    let connection = raw(path);
    let count = |kind: &str| {
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = ?1 AND name NOT LIKE 'sqlite_%'",
                [kind],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
    };
    let version = connection
        .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .unwrap();
    (version, count("table"), count("view"))
}

fn modified_a(repository: &TempDir) -> Capture {
    support::write(repository.path(), "a.ts", b"working\n");
    capture(repository.path(), &CaptureRequest::new(Selection::Local)).unwrap()
}

#[test]
fn retained_capture_survives_restart_and_loss_of_origins() {
    let repository = support::repository();
    let root = repository.path();
    let configuration_bytes = fs::read(root.join(".anti-slop.jsonc")).unwrap();
    support::write(root, "a.ts", b"working\n");
    let context_request = CaptureRequest {
        context: vec!["guide.ts".into()],
        ..CaptureRequest::new(Selection::Local)
    };
    let captured = capture(root, &context_request).unwrap();
    assert_eq!(captured.changes().len(), 1);
    assert_eq!(
        captured.changes()[0].before.as_ref().unwrap().bytes(),
        b"head\n"
    );
    assert_eq!(
        captured.changes()[0].after.as_ref().unwrap().bytes(),
        b"working\n"
    );
    assert_eq!(captured.context()[0].bytes(), b"guide\n");
    assert_eq!(captured.context()[0].role, Role::Context);
    assert_eq!(
        captured.changes()[0].after.as_ref().unwrap().role,
        Role::Working
    );
    assert_eq!(
        captured.resolved_commit(),
        Some(support::git(root, &["rev-parse", "HEAD"]).as_str())
    );
    let head = &captured.changes()[0].before.as_ref().unwrap();
    assert_eq!(
        head.git_blob.as_deref(),
        Some(support::git(root, &["rev-parse", "HEAD:a.ts"]).as_str())
    );
    assert_eq!(head.git_mode, Some(GitMode::Regular));
    let working = &captured.changes()[0].after.as_ref().unwrap();
    assert_eq!(working.git_blob, None);
    assert_eq!(working.git_mode, Some(GitMode::Regular));

    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    drop(store);

    let after = captured.changes()[0].after.as_ref().unwrap();
    let slice = SourceRange {
        path: after.path.clone(),
        role: after.role,
        generation: after.generation.clone(),
        range: 1..4,
    };
    let mut expected = captured_json(&captured, Some(support::hex(b"ork")));
    expected["id"] = json!(id.0);
    let digest = Sha256::digest(&configuration_bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(expected["configuration"]["source"]["sha256"], json!(digest));
    assert_eq!(
        expected["configuration"]["captured"],
        json!(support::hex(&configuration_bytes))
    );

    fs::remove_dir_all(root).unwrap();

    let out = directory.path().join("read.json");
    let child = support::spawn_probe(
        "read",
        &[
            ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
            ("AS_RECORDS_ID", id.0.to_string()),
            ("AS_RECORDS_OUT", out.to_string_lossy().into_owned()),
            (
                "AS_RECORDS_SLICE",
                format!(
                    "{}|{}|{}|{}|{}",
                    slice.path,
                    role_name(slice.role),
                    slice.generation,
                    slice.range.start,
                    slice.range.end
                ),
            ),
        ],
    );
    let mut child = child;
    let status = support::wait_bounded(&mut child, Duration::from_secs(30));
    assert!(status.success());
    let observed: Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert_eq!(observed, expected);
}

#[test]
fn retained_content_preserves_deletion_empty_and_equal_occurrences() {
    let repository = support::repository();
    let root = repository.path();
    fs::remove_file(root.join("removed.ts")).unwrap();
    support::write(root, "emptied.ts", b"");
    support::write(root, "equal-a.ts", b"same\n");
    support::write(root, "equal-b.ts", b"same\n");
    let captured = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
    assert_eq!(
        captured
            .changes()
            .iter()
            .map(|change| change.path.as_str())
            .collect::<Vec<_>>(),
        ["emptied.ts", "equal-a.ts", "equal-b.ts", "removed.ts"]
    );
    let emptied = &captured.changes()[0];
    assert_eq!(emptied.before.as_ref().unwrap().bytes(), b"filled\n");
    let emptied_after = emptied.after.as_ref().expect("empty file is present");
    assert!(emptied_after.bytes().is_empty());
    let removed = &captured.changes()[3];
    assert!(removed.after.is_none());
    assert_eq!(removed.before.as_ref().unwrap().bytes(), b"remove me\n");
    let equal_a = captured.changes()[1].after.as_ref().unwrap();
    let equal_b = captured.changes()[2].after.as_ref().unwrap();
    assert_eq!(equal_a.bytes(), equal_b.bytes());
    assert_ne!(equal_a.generation, equal_b.generation);

    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    let retained = store.read_capture(id).unwrap();
    assert_eq!(retained.changes().len(), 4);
    let retained_empty = &retained.changes()[0];
    assert_eq!(retained_empty.before.as_ref().unwrap().bytes(), b"filled\n");
    assert!(
        retained_empty
            .after
            .as_ref()
            .expect("empty file is present")
            .bytes()
            .is_empty()
    );
    assert!(retained.changes()[3].after.is_none());
    assert_eq!(
        retained.changes()[1].after.as_ref().unwrap().generation,
        equal_a.generation
    );
    assert_eq!(
        retained.changes()[2].after.as_ref().unwrap().generation,
        equal_b.generation
    );

    let empty_range = SourceRange {
        path: emptied_after.path.clone(),
        role: emptied_after.role,
        generation: emptied_after.generation.clone(),
        range: 0..0,
    };
    assert_eq!(retained.slice(&empty_range).unwrap(), b"");
}

#[test]
fn empty_selection_round_trips_with_its_identities() {
    let repository = support::repository();
    let captured = capture(repository.path(), &CaptureRequest::new(Selection::Local)).unwrap();
    assert!(captured.changes().is_empty());
    assert!(captured.context().is_empty());

    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    let repeated = store.save_capture(&captured).unwrap().id;
    assert_ne!(id, repeated);
    assert_eq!(
        store.read_capture(repeated).unwrap().generation(),
        captured.generation()
    );
    let retained = store.read_capture(id).unwrap();
    assert!(retained.changes().is_empty());
    assert!(retained.context().is_empty());
    assert_eq!(retained.selection(), &Selection::Local);
    assert_eq!(retained.generation(), captured.generation());
    assert_eq!(
        retained.configuration().captured_bytes(),
        captured.configuration().captured_bytes()
    );

    let base = capture(
        repository.path(),
        &CaptureRequest::new(Selection::Base("HEAD".into())),
    )
    .unwrap();
    assert!(base.changes().is_empty());
    let base_id = store.save_capture(&base).unwrap().id;
    let retained = store.read_capture(base_id).unwrap();
    assert_eq!(retained.selection(), &Selection::Base("HEAD".into()));
    assert_eq!(
        retained.resolved_commit(),
        Some(support::git(repository.path(), &["rev-parse", "HEAD"]).as_str())
    );

    fs::remove_file(repository.path().join(".anti-slop.jsonc")).unwrap();
    let built_in = capture(repository.path(), &CaptureRequest::new(Selection::Unstaged)).unwrap();
    assert!(matches!(
        built_in.configuration().source(),
        ConfigurationSource::BuiltIn
    ));
    let built_in_id = store.save_capture(&built_in).unwrap().id;
    let retained = store.read_capture(built_in_id).unwrap();
    assert!(matches!(
        retained.configuration().source(),
        ConfigurationSource::BuiltIn
    ));
    assert!(retained.configuration().captured_bytes().is_none());
}

#[test]
fn reopen_is_stable_and_incompatible_stores_are_rejected() {
    let repository = support::repository();
    let captured = modified_a(&repository);
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    drop(store);

    let store = Store::open(&database).unwrap();
    let journal: String = raw(&database)
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal, "wal");
    let retained = store.read_capture(id).unwrap();
    assert_eq!(retained.generation(), captured.generation());
    assert_eq!(
        retained.changes()[0].after.as_ref().unwrap().bytes(),
        b"working\n"
    );
    assert!(matches!(
        store.read_capture(CaptureId(9_999)),
        Err(StoreError::NotFound(_))
    ));
    let after = retained.changes()[0].after.as_ref().unwrap();
    let unbound = SourceRange {
        path: after.path.clone(),
        role: after.role,
        generation: "not-the-retained-generation".into(),
        range: 0..1,
    };
    assert!(matches!(
        retained.slice(&unbound),
        Err(StoreError::UnboundRange)
    ));
    let wrong_role = SourceRange {
        path: after.path.clone(),
        role: Role::Head,
        generation: after.generation.clone(),
        range: 0..1,
    };
    assert!(matches!(
        retained.slice(&wrong_role),
        Err(StoreError::UnboundRange)
    ));
    let wrong_path = SourceRange {
        path: "elsewhere.ts".into(),
        role: after.role,
        generation: after.generation.clone(),
        range: 0..1,
    };
    assert!(matches!(
        retained.slice(&wrong_path),
        Err(StoreError::UnboundRange)
    ));
    drop(store);

    raw(&database)
        .execute_batch("PRAGMA user_version = 2")
        .unwrap();
    assert!(matches!(
        Store::open(&database),
        Err(StoreError::UnsupportedSchema {
            found: 2,
            supported: 1
        })
    ));

    let unrelated = directory.path().join("unrelated.sqlite3");
    raw(&unrelated)
        .execute_batch("CREATE TABLE notes (body TEXT)")
        .unwrap();
    assert!(matches!(
        Store::open(&unrelated),
        Err(StoreError::NotACaptureStore)
    ));

    let corrupt = directory.path().join("corrupt.sqlite3");
    fs::write(&corrupt, b"this is not a database").unwrap();
    assert!(matches!(Store::open(&corrupt), Err(StoreError::Corrupt(_))));

    assert!(matches!(
        Store::open(&directory.path().join("missing/records.sqlite3")),
        Err(StoreError::Unavailable(_))
    ));
}

#[test]
fn failed_schema_initialization_leaves_no_partial_store() {
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    raw(&database)
        .execute_batch("CREATE VIEW captures AS SELECT 1 AS id")
        .unwrap();
    assert!(matches!(Store::open(&database), Err(StoreError::Failed(_))));
    let (version, tables, views) = raw_schema(&database);
    assert_eq!(version, 0);
    assert_eq!(tables, 0);
    assert_eq!(views, 1);
}

#[test]
fn injected_write_failure_leaves_no_partial_capture() {
    let repository = support::repository();
    let root = repository.path();
    support::write(root, "a.ts", b"first\n");
    let first = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let first_id = store.save_capture(&first).unwrap().id;
    assert_eq!(raw_counts(&database), (1, 1, 2));

    raw(&database)
        .execute_batch(
            "CREATE TRIGGER injected_failure BEFORE INSERT ON capture_sources \
             WHEN NEW.ordinal = 1 BEGIN SELECT RAISE(ABORT, 'injected write failure'); END",
        )
        .unwrap();
    support::write(root, "a.ts", b"second\n");
    let second = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
    assert!(matches!(
        store.save_capture(&second),
        Err(StoreError::Failed(_))
    ));
    assert_eq!(raw_counts(&database), (1, 1, 2));
    assert_eq!(
        store.read_capture(first_id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"first\n"
    );

    raw(&database)
        .execute_batch("DROP TRIGGER injected_failure")
        .unwrap();
    let second_id = store.save_capture(&second).unwrap().id;
    assert_ne!(first_id, second_id);
    assert_eq!(raw_counts(&database), (2, 2, 4));
    assert_eq!(
        store.read_capture(second_id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"second\n"
    );
}

#[test]
fn writer_contention_is_bounded_and_reported() {
    let repository = support::repository();
    let captured = modified_a(&repository);
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let holder = raw(&database);
    holder.execute_batch("BEGIN IMMEDIATE").unwrap();

    let started = Instant::now();
    let error = store.save_capture(&captured).unwrap_err();
    let elapsed = started.elapsed();
    assert!(matches!(error, StoreError::Busy), "{error:?}");
    assert!(elapsed >= BUSY_TIMEOUT.mul_f32(0.8), "{elapsed:?}");
    assert!(
        elapsed < BUSY_TIMEOUT + Duration::from_secs(10),
        "{elapsed:?}"
    );
    assert_eq!(raw_counts(&database), (0, 0, 0));
    holder.execute_batch("ROLLBACK").unwrap();

    let id = store.save_capture(&captured).unwrap().id;
    assert_eq!(
        store.read_capture(id).unwrap().generation(),
        captured.generation()
    );
}

#[test]
fn committed_capture_survives_lost_acknowledgement() {
    let repository = support::repository();
    support::write(repository.path(), "a.ts", b"working\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let id_file = directory.path().join("id");
    let acknowledgement = directory.path().join("acknowledgement");
    let mut child = support::spawn_probe(
        "save-stall",
        &[
            (
                "AS_RECORDS_REPO",
                repository.path().to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
            ("AS_RECORDS_SELECTION", "local".into()),
            ("AS_RECORDS_ID_FILE", id_file.to_string_lossy().into_owned()),
            (
                "AS_RECORDS_ACK_FILE",
                acknowledgement.to_string_lossy().into_owned(),
            ),
        ],
    );
    if !support::wait_for(&id_file, Duration::from_secs(30)) {
        child.kill().unwrap();
        panic!("writer did not commit within the deadline");
    }
    let id: i64 = fs::read_to_string(&id_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    child.kill().unwrap();
    let _ = child.wait();
    assert!(!acknowledgement.exists());

    let store = Store::open(&database).unwrap();
    let retained = store.read_capture(CaptureId(id)).unwrap();
    assert_eq!(
        retained.changes()[0].after.as_ref().unwrap().bytes(),
        b"working\n"
    );
}

#[test]
fn writer_interrupted_before_commit_leaves_an_absent_capture() {
    let repository = support::repository();
    support::write(repository.path(), "a.ts", b"interrupted\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let ready = directory.path().join("ready");
    let go = directory.path().join("go");
    let mut child = support::spawn_probe(
        "save-ready",
        &[
            (
                "AS_RECORDS_REPO",
                repository.path().to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
            ("AS_RECORDS_SELECTION", "local".into()),
            (
                "AS_RECORDS_READY_FILE",
                ready.to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_GO_FILE", go.to_string_lossy().into_owned()),
        ],
    );
    if !support::wait_for(&ready, Duration::from_secs(30)) {
        child.kill().unwrap();
        panic!("writer did not open the store within the deadline");
    }
    let holder = raw(&database);
    holder.execute_batch("BEGIN IMMEDIATE").unwrap();
    fs::write(&go, b"go").unwrap();
    thread::sleep(Duration::from_millis(400));
    child.kill().unwrap();
    let _ = child.wait();
    holder.execute_batch("ROLLBACK").unwrap();
    assert_eq!(raw_counts(&database), (0, 0, 0));

    let captured = capture(repository.path(), &CaptureRequest::new(Selection::Local)).unwrap();
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    assert_eq!(raw_counts(&database), (1, 1, 2));
    assert_eq!(
        store.read_capture(id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"interrupted\n"
    );
}

/// Saves one real capture, proves it reads back intact, then applies a corruption
/// and returns the outcome of the next public read.
fn corrupted_read(mutation: impl FnOnce(&Path, i64)) -> StoreError {
    let repository = support::repository();
    support::write(repository.path(), "a.ts", b"after\n");
    let captured = capture(
        repository.path(),
        &CaptureRequest {
            context: vec!["guide.ts".into()],
            ..CaptureRequest::new(Selection::Local)
        },
    )
    .unwrap();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    let baseline = store.read_capture(id).unwrap();
    assert_eq!(
        baseline.changes()[0].after.as_ref().unwrap().bytes(),
        b"after\n"
    );
    assert_eq!(baseline.context()[0].bytes(), b"guide\n");
    drop(store);

    mutation(&database, id.0);

    let store = Store::open(&database).unwrap();
    match store.read_capture(id) {
        Ok(_) => panic!("corrupted retained evidence was returned as a complete capture"),
        Err(error) => error,
    }
}

#[test]
fn missing_retained_source_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch("DELETE FROM capture_sources WHERE side = 'after'")
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn missing_retained_change_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch("DELETE FROM capture_changes WHERE ordinal = 0")
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn altered_retained_bytes_are_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch(
                "UPDATE capture_sources SET bytes = CAST('xxxxx' || char(10) AS BLOB) \
                 WHERE side = 'after'",
            )
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn altered_source_provenance_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch(
                "UPDATE capture_sources SET role = 'head', generation = 'forged' \
                 WHERE side = 'after'",
            )
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn duplicate_retained_source_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch(
                "INSERT INTO capture_sources (
                    capture_id, ordinal, change_ordinal, side, path, role, generation,
                    git_blob, git_mode, bytes
                 ) SELECT capture_id, 999, change_ordinal, side, path, role, generation,
                          git_blob, git_mode, bytes
                 FROM capture_sources WHERE side = 'after'",
            )
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn extra_retained_membership_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch(
                "INSERT INTO capture_changes (capture_id, ordinal, path)
                 SELECT capture_id, 999, 'extra.ts' FROM capture_changes LIMIT 1",
            )
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn altered_capture_provenance_is_an_explicit_corruption() {
    let error = corrupted_read(|database, _| {
        raw(database)
            .execute_batch("UPDATE captures SET capture_generation = 'forged'")
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

#[test]
fn altered_configuration_bytes_are_an_explicit_corruption() {
    let error = corrupted_read(|database, id| {
        let connection = raw(database);
        let original: Vec<u8> = connection
            .query_row(
                "SELECT configuration_bytes FROM captures WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        let mut altered = original;
        altered[0] = b' ';
        connection
            .execute(
                "UPDATE captures SET configuration_bytes = ?1 WHERE id = ?2",
                rusqlite::params![altered, id],
            )
            .unwrap();
    });
    assert!(matches!(error, StoreError::Corrupt(_)), "{error:?}");
}

fn file_bytes(path: &Path) -> u64 {
    fs::metadata(path).map_or(0, |metadata| metadata.len())
}

fn storage_files(database: &Path) -> u64 {
    let mut total = file_bytes(database);
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut candidate = database.as_os_str().to_owned();
        candidate.push(suffix);
        total += file_bytes(Path::new(&candidate));
    }
    total
}

fn capture_after(root: &Path, bytes: &[u8]) -> Capture {
    support::write(root, "a.ts", bytes);
    capture(root, &CaptureRequest::new(Selection::Local)).unwrap()
}

#[test]
fn retained_usage_counts_database_and_auxiliary_files() {
    let repository = support::repository();
    let captured = capture_after(repository.path(), b"usage\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    assert_eq!(store.usage().unwrap().limit, DEFAULT_BUDGET_BYTES);
    assert!(!store.usage().unwrap().warning);
    let saved = store.save_capture(&captured).unwrap();
    let id = saved.id;
    let usage = store.usage().unwrap();
    assert!(usage.retained > 0);
    assert_eq!(saved.usage, usage);
    assert_eq!(usage.retained, storage_files(&database));
    assert_eq!(usage.limit, DEFAULT_BUDGET_BYTES);
    assert!(!usage.warning);
    assert!(!usage.over_limit);
    assert_eq!(
        store.read_capture(id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"usage\n"
    );
}

#[test]
fn warning_starts_at_the_eighty_percent_line() {
    let repository = support::repository();
    let captured = capture_after(repository.path(), b"warning\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&captured).unwrap();
    drop(store);

    let store = Store::open(&database).unwrap();
    let retained = store.usage().unwrap().retained;
    assert!(retained > 0);
    drop(store);

    let at_line = u64::try_from((u128::from(retained) * 5).div_ceil(4)).unwrap();
    let store = Store::open_with_budget(&database, Budget::new(at_line)).unwrap();
    let warning = store.usage().unwrap();
    assert_eq!(warning.limit, at_line);
    assert!(warning.warning, "{warning:?} from retained {retained}");
    drop(store);

    let below = retained.saturating_mul(2).max(retained + 64 * 1024);
    let store = Store::open_with_budget(&database, Budget::new(below)).unwrap();
    assert!(!store.usage().unwrap().warning);
}

#[test]
fn budget_refusal_preserves_prior_records_and_reads() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"first\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let first_id = store.save_capture(&first).unwrap().id;
    drop(store);

    let mut store = Store::open_with_budget(&database, Budget::new(1)).unwrap();
    let usage = store.usage().unwrap();
    assert!(usage.retained > usage.limit);
    assert!(usage.warning);
    assert!(usage.over_limit);
    assert_eq!(
        store.read_capture(first_id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"first\n"
    );
    let second = capture_after(repository.path(), b"second\n");
    let counts = raw_counts(&database);
    assert!(matches!(
        store.save_capture(&second),
        Err(StoreError::BudgetExceeded { .. })
    ));
    assert_eq!(raw_counts(&database), counts);
    assert_eq!(
        store.read_capture(first_id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"first\n"
    );
}

#[test]
fn admission_boundary_matches_the_reported_requirement() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"first\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&first).unwrap();
    drop(store);

    let second = capture_after(repository.path(), b"second\n");
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let required = match probe.save_capture(&second) {
        Err(StoreError::BudgetExceeded { required, .. }) => required,
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);
    assert!(required > 0);

    let mut below = Store::open_with_budget(&database, Budget::new(required - 1)).unwrap();
    assert!(matches!(
        below.save_capture(&second),
        Err(StoreError::BudgetExceeded { .. })
    ));
    drop(below);

    let mut store = Store::open_with_budget(&database, Budget::new(required)).unwrap();
    let id = store.save_capture(&second).unwrap().id;
    let usage = store.usage().unwrap();
    assert_eq!(usage.limit, required);
    assert!(
        usage.retained <= required,
        "retained {} exceeded admitted requirement {required}",
        usage.retained
    );
    assert_eq!(
        store.read_capture(id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"second\n"
    );
}

#[test]
fn source_lengths_alone_do_not_qualify_admission() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"first\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&first).unwrap();
    drop(store);

    let second = capture_after(repository.path(), b"x\n");
    let payload = second.stats().retained_bytes as u64;
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let (retained, required) = match probe.save_capture(&second) {
        Err(StoreError::BudgetExceeded {
            retained, required, ..
        }) => (retained, required),
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);
    assert!(
        required - retained > payload,
        "reservation {} did not exceed payload {payload}",
        required - retained
    );
}

#[test]
fn two_processes_cannot_both_admit_one_capture() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"baseline\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let first_id = store.save_capture(&first).unwrap().id;
    drop(store);

    let contested = capture_after(repository.path(), b"contested\n");
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let required = match probe.save_capture(&contested) {
        Err(StoreError::BudgetExceeded { required, .. }) => required,
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);

    let start = directory.path().join("start");
    let outcomes = [
        directory.path().join("a.out"),
        directory.path().join("b.out"),
    ];
    let mut children = Vec::new();
    for (index, outcome) in outcomes.iter().enumerate() {
        let ready = directory.path().join(format!("ready-{index}"));
        children.push(support::spawn_probe(
            "save-budget",
            &[
                (
                    "AS_RECORDS_REPO",
                    repository.path().to_string_lossy().into_owned(),
                ),
                ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
                ("AS_RECORDS_SELECTION", "local".into()),
                ("AS_RECORDS_BUDGET", required.to_string()),
                (
                    "AS_RECORDS_READY_FILE",
                    ready.to_string_lossy().into_owned(),
                ),
                (
                    "AS_RECORDS_START_FILE",
                    start.to_string_lossy().into_owned(),
                ),
                ("AS_RECORDS_OUT", outcome.to_string_lossy().into_owned()),
            ],
        ));
    }
    for index in 0..children.len() {
        let ready = directory.path().join(format!("ready-{index}"));
        assert!(support::wait_for(&ready, Duration::from_secs(30)));
    }
    fs::write(&start, b"go").unwrap();
    for child in &mut children {
        let status = support::wait_bounded(child, Duration::from_secs(30));
        assert!(status.success());
    }
    let observed = outcomes
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        observed
            .iter()
            .filter(|line| line.starts_with("saved:"))
            .count(),
        1,
        "{observed:?}"
    );
    let refused = observed
        .iter()
        .find(|line| line.starts_with("refused:"))
        .unwrap_or_else(|| panic!("expected one refused contender: {observed:?}"));
    assert!(
        refused.contains("Collection budget"),
        "contender was refused for a reason other than the budget: {refused}"
    );
    assert_eq!(raw_counts(&database).0, 2);
    let store = Store::open(&database).unwrap();
    assert_eq!(
        store.read_capture(first_id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"baseline\n"
    );
}

#[test]
fn new_database_files_follow_the_effective_parent() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let real = directory.path().join("real");
    fs::create_dir(&real).unwrap();
    let alias_directory = directory.path().join("alias-dir");
    std::os::unix::fs::symlink(&real, &alias_directory).unwrap();
    let alias_database = alias_directory.join("new.sqlite3");
    let mut store = Store::open_with_budget(&alias_database, Budget::new(4 * 1024 * 1024))
        .expect("new database opens through the symlinked parent");
    let captured = capture_after(repository.path(), b"new\n");
    let saved = store.save_capture(&captured).unwrap();
    let real_database = real.join("new.sqlite3");
    assert!(real_database.is_file());
    assert_eq!(saved.usage.retained, storage_files(&real_database));
    assert_eq!(store.usage().unwrap(), saved.usage);
    assert!(alias_database.is_file());
    assert_eq!(
        store.read_capture(saved.id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"new\n"
    );
}

#[test]
fn dangling_database_symlink_creates_the_target_identity() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let real = directory.path().join("real");
    fs::create_dir(&real).unwrap();
    let target = real.join("target.sqlite3");
    let alias = directory.path().join("dangling.sqlite3");
    std::os::unix::fs::symlink(&target, &alias).unwrap();
    let mut store = Store::open_with_budget(&alias, Budget::new(4 * 1024 * 1024))
        .expect("dangling symlink opens the target database");
    let captured = capture_after(repository.path(), b"dangling\n");
    let saved = store.save_capture(&captured).unwrap();
    assert!(target.is_file());
    assert_eq!(saved.usage.retained, storage_files(&target));
    assert_eq!(
        store.read_capture(saved.id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"dangling\n"
    );
}

fn stage_three_mib_batch(root: &Path, batch: usize) -> Capture {
    for file in 0..3 {
        support::write(
            root,
            &format!("batch-{batch}-{file}.bin"),
            &vec![b'a' + batch as u8; 1024 * 1024],
        );
    }
    support::git(root, &["add", "-A"]);
    capture(root, &CaptureRequest::new(Selection::Staged)).unwrap()
}

fn commit_three_mib_batch(root: &Path, batch: usize) {
    support::git(root, &["commit", "-qm", &format!("batch {batch}")]);
}

#[test]
fn quota_accounting_uses_the_effective_database_files() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let real = directory.path().join("real");
    fs::create_dir(&real).unwrap();
    let database = real.join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let reader = raw(&database);
    reader.execute_batch("BEGIN").unwrap();
    let _: i64 = reader
        .query_row("SELECT count(*) FROM captures", [], |row| row.get(0))
        .unwrap();
    for batch in 0..3 {
        let captured = stage_three_mib_batch(repository.path(), batch);
        store.save_capture(&captured).unwrap();
        commit_three_mib_batch(repository.path(), batch);
    }

    let file_alias = directory.path().join("alias.sqlite3");
    std::os::unix::fs::symlink(&database, &file_alias).unwrap();
    let directory_alias = directory.path().join("alias-dir");
    std::os::unix::fs::symlink(&real, &directory_alias).unwrap();

    for (batch, alias) in [
        (3, file_alias.clone()),
        (4, directory_alias.join("records.sqlite3")),
    ] {
        let canonical_before = storage_files(&database);
        assert!(canonical_before > 6 * 1024 * 1024, "{canonical_before}");
        let captured = stage_three_mib_batch(repository.path(), batch);

        let mut probe = Store::open_with_budget(&alias, Budget::new(0)).unwrap();
        let (retained, required) = match probe.save_capture(&captured) {
            Err(StoreError::BudgetExceeded {
                retained, required, ..
            }) => (retained, required),
            other => panic!("expected an explicit budget refusal, got {other:?}"),
        };
        drop(probe);

        let mut alias_store = Store::open_with_budget(&alias, Budget::new(required)).unwrap();
        let saved = alias_store.save_capture(&captured).unwrap();
        let canonical_after = storage_files(&database);
        assert_eq!(
            retained, canonical_before,
            "alias admission omitted effective database auxiliaries"
        );
        assert!(
            canonical_after <= required,
            "committed footprint {canonical_after} exceeded admitted {required}"
        );
        assert_eq!(
            saved.usage.retained, canonical_after,
            "post-commit usage did not measure the effective database"
        );
        assert!(!saved.usage.over_limit);
        eprintln!(
            "alias batch={batch} retained={retained} canonical_before={canonical_before} \
             required={required} canonical_after={canonical_after} post_commit={}",
            saved.usage.retained
        );
        let retained_bytes = alias_store
            .read_capture(saved.id)
            .unwrap()
            .changes()
            .iter()
            .filter_map(|change| change.after.as_ref())
            .map(|source| source.bytes().len())
            .sum::<usize>();
        assert_eq!(retained_bytes, 3 * 1024 * 1024);
        commit_three_mib_batch(repository.path(), batch);
    }
    reader.execute_batch("ROLLBACK").unwrap();
}

fn spawn_fault_child(
    repository: &Path,
    database: &Path,
    directory: &Path,
    mode: &str,
    budget: Option<u64>,
    extra: Option<&Path>,
) -> (PathBuf, std::process::Child) {
    let library = support::build_io_fault_interposer(directory);
    let enable = directory.join("fault-enable");
    let out = directory.join(format!("fault-{mode}.out"));
    let mut variables = vec![
        ("LD_PRELOAD", library.to_string_lossy().into_owned()),
        ("AS_IO_FAULT", mode.to_owned()),
        ("AS_FAIL_ENABLE", enable.to_string_lossy().into_owned()),
        ("AS_FAULT_DB", database.to_string_lossy().into_owned()),
        ("AS_RECORDS_REPO", repository.to_string_lossy().into_owned()),
        ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
        ("AS_RECORDS_OUT", out.to_string_lossy().into_owned()),
    ];
    if let Some(extra) = extra {
        variables.push(("AS_FAULT_EXTRA", extra.to_string_lossy().into_owned()));
    }
    if let Some(budget) = budget {
        variables.push(("AS_RECORDS_BUDGET", budget.to_string()));
    }
    let child = support::spawn_probe("fault-commit", &variables);
    (out, child)
}

fn fault_report(out: &Path) -> Vec<String> {
    fs::read_to_string(out)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

#[test]
fn process_killed_during_commit_is_complete_or_absent() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let library = support::build_io_fault_interposer(directory.path());
    let enable = directory.path().join("fault-enable");
    let ready = directory.path().join("fault-ready");
    let out = directory.path().join("fault-pause.out");
    let mut child = support::spawn_probe(
        "fault-commit",
        &[
            ("LD_PRELOAD", library.to_string_lossy().into_owned()),
            ("AS_IO_FAULT", "pause".into()),
            ("AS_FAIL_ENABLE", enable.to_string_lossy().into_owned()),
            ("AS_PAUSE_READY", ready.to_string_lossy().into_owned()),
            ("AS_FAULT_DB", database.to_string_lossy().into_owned()),
            (
                "AS_RECORDS_REPO",
                repository.path().to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
            ("AS_RECORDS_OUT", out.to_string_lossy().into_owned()),
        ],
    );
    assert!(
        support::wait_for(&ready, Duration::from_secs(30)),
        "writer never reached the WAL synchronization seam"
    );
    let baseline_id: i64 = fs::read_to_string(&out)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("baseline:"))
        .expect("baseline id before pause")
        .parse()
        .unwrap();
    child.kill().unwrap();
    let _ = child.wait();

    let store = Store::open(&database).unwrap();
    assert_eq!(
        store
            .read_capture(CaptureId(baseline_id))
            .unwrap()
            .changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"baseline\n"
    );
    let counts = raw_counts(&database);
    let outcome = if counts.0 == 1 {
        assert_eq!(counts, (1, 1, 1));
        "absent".to_owned()
    } else {
        assert_eq!(counts.0, 2, "unexpected committed capture count {counts:?}");
        let attempt = store
            .read_capture(CaptureId(baseline_id + 1))
            .expect("a present capture must pass the retained integrity read");
        assert_eq!(attempt.changes().len(), 2);
        let attempt_change = attempt
            .changes()
            .iter()
            .find(|change| change.path == "fault-attempt.txt")
            .expect("attempt source retained");
        assert_eq!(attempt_change.after.as_ref().unwrap().bytes(), b"attempt\n");
        "complete".to_owned()
    };
    eprintln!("during-commit kill outcome: {outcome}; counts={counts:?}");
}

#[test]
fn read_only_database_storage_fails_explicitly() {
    use std::os::unix::fs::PermissionsExt;

    let repository = support::repository();
    let captured = capture_after(repository.path(), b"readonly\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&captured).unwrap();
    drop(store);

    fs::set_permissions(&database, fs::Permissions::from_mode(0o444)).unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let result = Store::open(&database);
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&database, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        matches!(result, Err(StoreError::Unavailable(_))),
        "{:?}",
        result.err()
    );
}

#[test]
fn commit_io_failure_keeps_acknowledged_records_and_no_partial_capture() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let (out, mut child) = spawn_fault_child(
        repository.path(),
        &database,
        directory.path(),
        "write",
        None,
        None,
    );
    let status = support::wait_bounded(&mut child, Duration::from_secs(30));
    assert!(status.success());
    let report = fault_report(&out);
    let baseline_id: i64 = report
        .iter()
        .find_map(|line| line.strip_prefix("baseline:"))
        .expect("baseline report")
        .parse()
        .unwrap();
    let attempt = report
        .iter()
        .find_map(|line| line.strip_prefix("attempt:"))
        .expect("attempt report");
    eprintln!("controlled commit failure: {attempt}");
    assert!(attempt.starts_with("error:"), "{attempt}");
    let lowered = attempt.to_lowercase();
    assert!(
        lowered.contains("i/o") || lowered.contains("failed"),
        "{attempt}"
    );
    assert_eq!(raw_counts(&database), (1, 1, 1));

    let store = Store::open(&database).unwrap();
    assert_eq!(
        store
            .read_capture(CaptureId(baseline_id))
            .unwrap()
            .changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"baseline\n"
    );
    assert_eq!(raw_counts(&database), (1, 1, 1));
}

#[test]
fn postcommit_over_limit_is_reported_with_committed_identity() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let extra = PathBuf::from(format!("{}-journal", database.display()));
    let limit = 1_500_000;
    let (out, mut child) = spawn_fault_child(
        repository.path(),
        &database,
        directory.path(),
        "over",
        Some(limit),
        Some(&extra),
    );
    let status = support::wait_bounded(&mut child, Duration::from_secs(30));
    assert!(status.success());
    let report = fault_report(&out);
    let attempt = report
        .iter()
        .find_map(|line| line.strip_prefix("attempt:"))
        .expect("attempt report");
    eprintln!("postcommit over-limit result: {attempt}");
    assert!(attempt.starts_with("saved:"), "{attempt}");
    let fields = attempt
        .strip_prefix("saved:")
        .unwrap()
        .split(':')
        .collect::<Vec<_>>();
    let id: i64 = fields[0].parse().unwrap();
    assert_eq!(fields[1], "over=true", "{attempt}");
    let retained: u64 = fields[2]
        .strip_prefix("retained=")
        .unwrap()
        .parse()
        .unwrap();
    assert!(retained > limit, "{attempt}");
    assert!(extra.is_file());

    let store = Store::open_with_budget(&database, Budget::new(limit)).unwrap();
    let retained_capture = store.read_capture(CaptureId(id)).unwrap();
    let attempt_change = retained_capture
        .changes()
        .iter()
        .find(|change| change.path == "fault-attempt.txt")
        .expect("committed attempt");
    assert_eq!(attempt_change.after.as_ref().unwrap().bytes(), b"attempt\n");
}

#[test]
fn postcommit_measurement_failure_preserves_the_committed_record() {
    let repository = support::repository();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let (out, mut child) = spawn_fault_child(
        repository.path(),
        &database,
        directory.path(),
        "unmeasure",
        None,
        None,
    );
    let status = support::wait_bounded(&mut child, Duration::from_secs(30));
    assert!(status.success());
    let report = fault_report(&out);
    let attempt = report
        .iter()
        .find_map(|line| line.strip_prefix("attempt:"))
        .expect("attempt report");
    eprintln!("postcommit measurement failure: {attempt}");
    assert!(attempt.starts_with("error:"), "{attempt}");
    assert!(attempt.contains("was committed"), "{attempt}");
    let id: i64 = attempt
        .split_whitespace()
        .find_map(|word| word.parse::<i64>().ok())
        .expect("committed capture id");
    let moved = PathBuf::from(format!("{}.moved", database.display()));
    assert!(moved.is_file());
    fs::rename(&moved, &database).unwrap();

    let store = Store::open(&database).unwrap();
    let retained = store.read_capture(CaptureId(id)).unwrap();
    assert!(
        retained
            .changes()
            .iter()
            .any(|change| change.path == "fault-attempt.txt")
    );
    assert_eq!(
        retained
            .changes()
            .iter()
            .find(|change| change.path == "fault-attempt.txt")
            .and_then(|change| change.after.as_ref())
            .unwrap()
            .bytes(),
        b"attempt\n"
    );
}

fn large_repository() -> TempDir {
    let repository = support::repository();
    for index in 0..8 {
        support::write(
            repository.path(),
            &format!("large-{index}.bin"),
            &vec![b'x'; 1024 * 1024],
        );
    }
    repository
}

#[test]
fn large_capture_peak_stays_within_admitted_budget() {
    let repository = large_repository();
    let captured = capture(repository.path(), &CaptureRequest::new(Selection::Local)).unwrap();
    let payload = captured.stats().retained_bytes as u64;
    assert!(payload >= 8 * 1024 * 1024);
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let (retained, required) = match probe.save_capture(&captured) {
        Err(StoreError::BudgetExceeded {
            retained, required, ..
        }) => (retained, required),
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);
    let reservation = required - retained;
    assert!(
        reservation > payload,
        "reservation {reservation} did not exceed payload {payload}"
    );

    let sampling = Arc::new(AtomicBool::new(true));
    let peak = Arc::new(AtomicU64::new(0));
    let monitor_flag = Arc::clone(&sampling);
    let monitor_peak = Arc::clone(&peak);
    let monitor_database = database.clone();
    let monitor = thread::spawn(move || {
        let mut samples = 0_u64;
        while monitor_flag.load(Ordering::Relaxed) {
            monitor_peak.fetch_max(storage_files(&monitor_database), Ordering::Relaxed);
            samples += 1;
            thread::sleep(Duration::from_millis(1));
        }
        samples
    });

    let mut store = Store::open_with_budget(&database, Budget::new(required)).unwrap();
    let id = store.save_capture(&captured).unwrap().id;
    sampling.store(false, Ordering::Relaxed);
    let samples = monitor.join().unwrap();
    let usage = store.usage().unwrap();
    let peak = peak.load(Ordering::Relaxed);
    eprintln!(
        "payload={payload} retained_before={retained} reservation={reservation} \
         required={required} final={} peak={peak} samples={samples}",
        usage.retained
    );
    assert!(
        usage.retained <= required,
        "final retained {} exceeded {required}",
        usage.retained
    );
    assert!(peak <= required, "peak {peak} exceeded {required}");
    let retained_capture = store.read_capture(id).unwrap();
    let large = retained_capture
        .changes()
        .iter()
        .find(|change| change.path == "large-0.bin")
        .and_then(|change| change.after.as_ref())
        .expect("large addition retained");
    assert_eq!(large.bytes().len(), 1024 * 1024);
    assert!(large.bytes().iter().all(|byte| *byte == b'x'));
}

#[test]
fn repeated_large_writes_remain_admitted_and_accounted() {
    let repository = support::repository();
    let root = repository.path();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut previous = 0;
    let mut ids = Vec::new();

    for (index, fill) in [b'a', b'b'].into_iter().enumerate() {
        let name = format!("repeat-{index}.bin");
        support::write(root, &name, &vec![fill; 1024 * 1024]);
        let captured = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
        let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
        let required = match probe.save_capture(&captured) {
            Err(StoreError::BudgetExceeded { required, .. }) => required,
            other => panic!("expected an explicit budget refusal, got {other:?}"),
        };
        drop(probe);
        let mut store = Store::open_with_budget(&database, Budget::new(required)).unwrap();
        let before = store.usage().unwrap().retained;
        let id = store.save_capture(&captured).unwrap().id;
        let after = store.usage().unwrap();
        assert!(after.retained <= required, "{after:?} exceeded {required}");
        assert!(after.retained >= before);
        if index > 0 {
            assert!(after.retained >= previous);
        }
        previous = after.retained;
        assert_eq!(
            store
                .read_capture(id)
                .unwrap()
                .changes()
                .last()
                .map(|c| c.path.as_str()),
            Some(name.as_str())
        );
        ids.push(id);
        drop(store);
        support::git(root, &["add", &name]);
        support::git(root, &["commit", "-qm", &format!("add {name}")]);
    }
    let store = Store::open(&database).unwrap();
    for id in ids {
        assert!(store.read_capture(id).is_ok());
    }
}

#[test]
fn repeated_writes_and_checkpoints_remain_accounted() {
    let repository = support::repository();
    let root = repository.path();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let limit = 64 * 1024 * 1024;
    let mut store = Store::open_with_budget(&database, Budget::new(limit)).unwrap();
    let mut ids = Vec::new();
    let mut usages = Vec::new();
    for index in 0..6 {
        let name = format!("checkpoint-{index}.bin");
        support::write(root, &name, &vec![b'a' + index as u8; 1024 * 1024]);
        let captured = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
        ids.push(store.save_capture(&captured).unwrap().id);
        usages.push(store.usage().unwrap().retained);
        support::git(root, &["add", &name]);
        support::git(root, &["commit", "-qm", &format!("add {name}")]);
    }
    eprintln!("checkpoint usages: {usages:?}");
    assert!(usages.iter().all(|usage| *usage <= limit));
    assert!(usages.last().unwrap() > &(4 * 1024 * 1024));
    for id in ids {
        assert!(store.read_capture(id).is_ok());
    }
}

#[test]
fn reservation_accounts_for_the_existing_checkpoint_backlog() {
    let repository = support::repository();
    let root = repository.path();
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut writer = Store::open(&database).unwrap();
    for index in 0..3 {
        let name = format!("backlog-{index}.bin");
        support::write(root, &name, &vec![b'b'; 1024 * 1024]);
        let captured = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
        writer.save_capture(&captured).unwrap();
        support::git(root, &["add", &name]);
        support::git(root, &["commit", "-qm", &format!("add {name}")]);
    }
    support::write(root, "tiny.txt", b"x\n");
    let tiny = capture(root, &CaptureRequest::new(Selection::Local)).unwrap();
    let wal_with_backlog = file_bytes(&suffixed_path(&database, "-wal"));
    assert!(wal_with_backlog > 2 * 1024 * 1024, "{wal_with_backlog}");

    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let (retained_with, required_with) = match probe.save_capture(&tiny) {
        Err(StoreError::BudgetExceeded {
            retained, required, ..
        }) => (retained, required),
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);
    drop(writer);

    let wal_without_backlog = file_bytes(&suffixed_path(&database, "-wal"));
    assert!(wal_without_backlog < wal_with_backlog);
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let (retained_without, required_without) = match probe.save_capture(&tiny) {
        Err(StoreError::BudgetExceeded {
            retained, required, ..
        }) => (retained, required),
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);
    assert_eq!(
        required_with - retained_with,
        (required_without - retained_without) + (wal_with_backlog - wal_without_backlog)
    );
}

fn suffixed_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

#[test]
fn interrupted_admission_leaks_no_capacity() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"first\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&first).unwrap();
    drop(store);

    let second = capture_after(repository.path(), b"second\n");
    let mut probe = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    let required = match probe.save_capture(&second) {
        Err(StoreError::BudgetExceeded { required, .. }) => required,
        other => panic!("expected an explicit budget refusal, got {other:?}"),
    };
    drop(probe);

    let ready = directory.path().join("ready");
    let go = directory.path().join("go");
    let mut child = support::spawn_probe(
        "save-ready",
        &[
            (
                "AS_RECORDS_REPO",
                repository.path().to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_DB", database.to_string_lossy().into_owned()),
            ("AS_RECORDS_SELECTION", "local".into()),
            ("AS_RECORDS_BUDGET", required.to_string()),
            (
                "AS_RECORDS_READY_FILE",
                ready.to_string_lossy().into_owned(),
            ),
            ("AS_RECORDS_GO_FILE", go.to_string_lossy().into_owned()),
        ],
    );
    assert!(support::wait_for(&ready, Duration::from_secs(30)));
    let holder = raw(&database);
    holder.execute_batch("BEGIN IMMEDIATE").unwrap();
    fs::write(&go, b"go").unwrap();
    thread::sleep(Duration::from_millis(400));
    child.kill().unwrap();
    let _ = child.wait();
    holder.execute_batch("ROLLBACK").unwrap();
    assert_eq!(raw_counts(&database).0, 1);

    let mut store = Store::open_with_budget(&database, Budget::new(required)).unwrap();
    let id = store.save_capture(&second).unwrap().id;
    assert!(store.usage().unwrap().retained <= required);
    assert_eq!(
        store.read_capture(id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"second\n"
    );
    let third = capture_after(repository.path(), b"third\n");
    assert!(matches!(
        store.save_capture(&third),
        Err(StoreError::BudgetExceeded { .. })
    ));
}

#[test]
fn unavailable_storage_accounting_blocks_admission() {
    let repository = support::repository();
    let first = capture_after(repository.path(), b"first\n");
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&first).unwrap();
    fs::remove_file(&database).unwrap();
    fs::create_dir(&database).unwrap();
    let second = capture_after(repository.path(), b"second\n");
    assert!(matches!(
        store.save_capture(&second),
        Err(StoreError::Unavailable(_))
    ));
    fs::remove_dir(&database).unwrap();
}

/// Fresh-process probe; every parent test selects it by exact name and supplies the mode.
#[test]
fn child_process_probe() {
    let Ok(mode) = std::env::var("AS_RECORDS_PROBE") else {
        return;
    };
    match mode.as_str() {
        "read" => {
            let database = PathBuf::from(std::env::var("AS_RECORDS_DB").unwrap());
            let id: i64 = std::env::var("AS_RECORDS_ID").unwrap().parse().unwrap();
            let out = PathBuf::from(std::env::var("AS_RECORDS_OUT").unwrap());
            let store = Store::open(&database).expect("fresh process opens the retained store");
            let retained = store
                .read_capture(CaptureId(id))
                .expect("fresh process reads the retained capture");
            let slice = std::env::var("AS_RECORDS_SLICE").ok().map(|spec| {
                let parts = spec.split('|').collect::<Vec<_>>();
                let range = SourceRange {
                    path: parts[0].to_owned(),
                    role: role_from_name(parts[1]),
                    generation: parts[2].to_owned(),
                    range: parts[3].parse().unwrap()..parts[4].parse().unwrap(),
                };
                support::hex(
                    retained
                        .slice(&range)
                        .expect("fresh process resolves the range"),
                )
            });
            fs::write(
                out,
                serde_json::to_vec(&retained_json(&retained, slice)).unwrap(),
            )
            .unwrap();
        }
        "fault-commit" => {
            let repository = PathBuf::from(std::env::var("AS_RECORDS_REPO").unwrap());
            let database = PathBuf::from(std::env::var("AS_RECORDS_DB").unwrap());
            let mut store = match std::env::var("AS_RECORDS_BUDGET") {
                Ok(budget) => Store::open_with_budget(
                    &database,
                    Budget::new(budget.parse().expect("numeric budget")),
                ),
                Err(_) => Store::open(&database),
            }
            .expect("fault writer opens the store");
            support::write(&repository, "fault-baseline.txt", b"baseline\n");
            let baseline = capture(&repository, &CaptureRequest::new(Selection::Local)).unwrap();
            let saved = store.save_capture(&baseline).expect("baseline commits");
            let mut report = format!("baseline:{}\n", saved.id.0);
            let out = PathBuf::from(std::env::var("AS_RECORDS_OUT").unwrap());
            fs::write(&out, &report).unwrap();
            if let Ok(enable) = std::env::var("AS_FAIL_ENABLE") {
                fs::write(enable, b"enabled").unwrap();
            }
            support::write(&repository, "fault-attempt.txt", b"attempt\n");
            let attempt = capture(&repository, &CaptureRequest::new(Selection::Local)).unwrap();
            match store.save_capture(&attempt) {
                Ok(saved) => report.push_str(&format!(
                    "attempt:saved:{}:over={}:retained={}\n",
                    saved.id.0, saved.usage.over_limit, saved.usage.retained
                )),
                Err(error) => report.push_str(&format!("attempt:error:{error}\n")),
            }
            fs::write(std::env::var("AS_RECORDS_OUT").unwrap(), report).unwrap();
        }
        "save-budget" => {
            let repository = PathBuf::from(std::env::var("AS_RECORDS_REPO").unwrap());
            let database = PathBuf::from(std::env::var("AS_RECORDS_DB").unwrap());
            let selection = selection_from_name(&std::env::var("AS_RECORDS_SELECTION").unwrap());
            let budget: u64 = std::env::var("AS_RECORDS_BUDGET").unwrap().parse().unwrap();
            let mut store =
                Store::open_with_budget(&database, Budget::new(budget)).expect("contender opens");
            fs::write(std::env::var("AS_RECORDS_READY_FILE").unwrap(), b"ready").unwrap();
            let start = PathBuf::from(std::env::var("AS_RECORDS_START_FILE").unwrap());
            assert!(support::wait_for(&start, Duration::from_secs(30)));
            let captured = capture(&repository, &CaptureRequest::new(selection)).unwrap();
            let outcome = match store.save_capture(&captured) {
                Ok(saved) => format!("saved:{}", saved.id),
                Err(error) => format!("refused:{error}"),
            };
            fs::write(std::env::var("AS_RECORDS_OUT").unwrap(), outcome).unwrap();
        }
        "save" | "save-stall" | "save-ready" => {
            let repository = PathBuf::from(std::env::var("AS_RECORDS_REPO").unwrap());
            let database = PathBuf::from(std::env::var("AS_RECORDS_DB").unwrap());
            let selection = selection_from_name(&std::env::var("AS_RECORDS_SELECTION").unwrap());
            let mut store = match std::env::var("AS_RECORDS_BUDGET") {
                Ok(budget) => Store::open_with_budget(
                    &database,
                    Budget::new(budget.parse().expect("numeric budget")),
                ),
                Err(_) => Store::open(&database),
            }
            .expect("writer opens the store");
            if mode == "save-ready" {
                fs::write(std::env::var("AS_RECORDS_READY_FILE").unwrap(), b"ready").unwrap();
                let go = PathBuf::from(std::env::var("AS_RECORDS_GO_FILE").unwrap());
                assert!(support::wait_for(&go, Duration::from_secs(30)));
            }
            let captured = capture(&repository, &CaptureRequest::new(selection)).unwrap();
            let id = store.save_capture(&captured).expect("writer commits").id;
            if let Ok(path) = std::env::var("AS_RECORDS_ID_FILE") {
                fs::write(path, id.0.to_string()).unwrap();
            }
            if mode == "save-stall" {
                thread::sleep(Duration::from_secs(60));
                fs::write(std::env::var("AS_RECORDS_ACK_FILE").unwrap(), b"ack").unwrap();
            }
        }
        other => panic!("unknown probe mode {other}"),
    }
}
