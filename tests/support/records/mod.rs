//! Real repository fixtures for the durable capture record tests.
#![allow(dead_code)]

use anti_slop::config::SCHEMA_URL;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

pub fn build_io_fault_interposer(directory: &Path) -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/records/io_fault.c");
    let library = directory.join("libas-io-fault.so");
    let output = Command::new("cc")
        .args(["-shared", "-fPIC", "-O2", "-o"])
        .arg(&library)
        .arg(&source)
        .arg("-ldl")
        .output()
        .expect("an existing C compiler is required for the test-only I/O interposer");
    assert!(
        output.status.success(),
        "building the test-only I/O interposer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    library
}

pub fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

pub fn repository() -> TempDir {
    let root = TempDir::new().unwrap();
    git(root.path(), &["init", "-q"]);
    write(root.path(), "a.ts", b"head\n");
    write(root.path(), "guide.ts", b"guide\n");
    write(root.path(), "removed.ts", b"remove me\n");
    write(root.path(), "emptied.ts", b"filled\n");
    write(root.path(), "equal-a.ts", b"unrelated\n");
    let configuration = serde_json::json!({
        "$schema": SCHEMA_URL,
        "rules": {"R4": "off"},
        "overrides": [{"include": ["*.ts"], "rules": {"R1": "block"}}],
    });
    write(
        root.path(),
        ".anti-slop.jsonc",
        configuration.to_string().as_bytes(),
    );
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "head"]);
    root
}

pub fn write(root: &Path, relative: &str, bytes: &[u8]) {
    fs::write(root.join(relative), bytes).unwrap();
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn wait_bounded(child: &mut Child, limit: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("child probe did not finish within {limit:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn wait_for(path: &Path, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

/// Runs one exact probe test in a fresh process; the parent supplies its payload through env.
pub fn spawn_probe(mode: &str, variables: &[(&str, String)]) -> Child {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .arg("--exact")
        .arg("child_process_probe")
        .arg("--nocapture")
        .env("AS_RECORDS_PROBE", mode)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in variables {
        command.env(name, value);
    }
    command.spawn().unwrap()
}
