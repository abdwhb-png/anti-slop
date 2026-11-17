//! One current nonvisual durable-capture journey for the PR demonstration.
//!
//! `cargo test --frozen --test records_demo -- --nocapture`

#[path = "support/records/mod.rs"]
mod support;

use anti_slop::{
    records::{Budget, CaptureId, Store, StoreError},
    workspace::{CaptureRequest, Selection, capture},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use tempfile::TempDir;

#[test]
fn demo_capture_save_read_and_quota_boundary() {
    let repository = support::repository();
    let root = repository.path();
    support::write(root, "a.ts", b"working selected\n");
    let captured = capture(
        root,
        &CaptureRequest {
            context: vec!["guide.ts".into()],
            ..CaptureRequest::new(Selection::Local)
        },
    )
    .unwrap();
    let change = &captured.changes()[0];
    let after = change.after.as_ref().unwrap();
    println!(
        "capture selection={:?} commit={} generation={}",
        captured.selection(),
        captured.resolved_commit().unwrap_or("-"),
        captured.generation()
    );
    println!(
        "selected path={} after_role={:?} before={} after={} context={}",
        change.path,
        after.role,
        support::hex(change.before.as_ref().unwrap().bytes()),
        support::hex(after.bytes()),
        support::hex(captured.context()[0].bytes())
    );

    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open_with_budget(&database, Budget::new(4 * 1024 * 1024)).unwrap();
    let saved = store.save_capture(&captured).unwrap();
    println!(
        "saved id={} retained={} limit={} warning={} over_limit={}",
        saved.id.0,
        saved.usage.retained,
        saved.usage.limit,
        saved.usage.warning,
        saved.usage.over_limit
    );

    support::write(root, "a.ts", b"working second\n");
    let second = capture(
        root,
        &CaptureRequest {
            context: vec!["guide.ts".into()],
            ..CaptureRequest::new(Selection::Local)
        },
    )
    .unwrap();
    let mut refused = Store::open_with_budget(&database, Budget::new(0)).unwrap();
    match refused.save_capture(&second) {
        Err(StoreError::BudgetExceeded {
            retained, required, ..
        }) => println!(
            "quota refused retained={retained} required={required} while prior id={} stays readable",
            saved.id.0
        ),
        other => panic!("expected quota refusal, got {other:?}"),
    }
    assert_eq!(
        refused.read_capture(saved.id).unwrap().changes()[0]
            .after
            .as_ref()
            .unwrap()
            .bytes(),
        b"working selected\n"
    );
    drop(refused);
    drop(store);

    fs::remove_dir_all(root).unwrap();
    println!("origins removed: {}", !root.exists());

    let out = directory.path().join("fresh-read.txt");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("demo_child_read")
        .arg("--nocapture")
        .env("AS_DEMO_DB", database.to_string_lossy().into_owned())
        .env("AS_DEMO_ID", saved.id.0.to_string())
        .env("AS_DEMO_OUT", out.to_string_lossy().into_owned())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let status = support::wait_bounded(&mut child, Duration::from_secs(20));
    assert!(status.success(), "fresh process read failed");
    let observed = fs::read_to_string(&out).unwrap();
    println!("fresh process read:\n{}", observed.trim());
    assert!(observed.contains(&support::hex(b"working selected\n")));
    assert!(observed.contains(&support::hex(b"guide\n")));
    assert!(observed.contains(&after.generation));
}

#[test]
fn demo_child_read() {
    let Ok(database) = std::env::var("AS_DEMO_DB") else {
        return;
    };
    let id: i64 = std::env::var("AS_DEMO_ID").unwrap().parse().unwrap();
    let out = PathBuf::from(std::env::var("AS_DEMO_OUT").unwrap());
    let store = Store::open(Path::new(&database)).unwrap();
    let retained = store.read_capture(CaptureId(id)).unwrap();
    let change = &retained.changes()[0];
    let after = change.after.as_ref().unwrap();
    let report = format!(
        "id={} selection={:?} commit={} generation={}\npath={} role={:?} bytes={} generation={}\ncontext_role={:?} context_bytes={}\n",
        retained.id().0,
        retained.selection(),
        retained.resolved_commit().unwrap_or("-"),
        retained.generation(),
        change.path,
        after.role,
        support::hex(after.bytes()),
        after.generation,
        retained.context()[0].role,
        support::hex(retained.context()[0].bytes())
    );
    fs::write(out, report).unwrap();
}
