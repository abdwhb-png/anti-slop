use anti_slop::workspace::{Role, StagedRequest, capture_staged};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
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
fn repository() -> TempDir {
    let root = TempDir::new().unwrap();
    git(root.path(), &["init", "-q"]);
    fs::write(root.path().join("a.ts"), b"head\n").unwrap();
    fs::write(root.path().join("staged-only.ts"), b"old\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "head"]);
    root
}

// Catches selecting disk bytes or silently including untracked/worktree-only inputs.
#[test]
fn staged_bytes_and_working_context_remain_independent() {
    let root = repository();
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    fs::write(root.path().join("staged-only.ts"), b"staged-only\n").unwrap();
    git(root.path(), &["add", "."]);
    fs::write(root.path().join("a.ts"), b"working\n").unwrap();
    fs::write(root.path().join("untracked.ts"), b"untracked\n").unwrap();
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["a.ts".into()],
            ..Default::default()
        },
    )
    .expect("staged capture must return retained HEAD/index and independent context");
    assert_eq!(capture.head(), git(root.path(), &["rev-parse", "HEAD"]));
    assert_eq!(
        capture
            .changes()
            .iter()
            .map(|c| c.path.as_str())
            .collect::<Vec<_>>(),
        ["a.ts", "staged-only.ts"]
    );
    let change = &capture.changes()[0];
    assert_eq!(change.before.as_ref().unwrap().role, Role::Head);
    assert_eq!(change.before.as_ref().unwrap().bytes(), b"head\n");
    assert_eq!(change.after.as_ref().unwrap().role, Role::Index);
    assert_eq!(change.after.as_ref().unwrap().bytes(), b"index\n");
    assert_eq!(capture.context()[0].role, Role::Context);
    assert_eq!(capture.context()[0].bytes(), b"working\n");
    fs::remove_file(root.path().join("a.ts")).unwrap();
    assert_eq!(capture.context()[0].bytes(), b"working\n");
    assert_eq!(change.after.as_ref().unwrap().bytes(), b"index\n");
}

// Catches reloading per path, mutable provenance and ranges that silently bind another view.
#[test]
fn retained_configuration_and_ranges_share_the_actual_generation() {
    use anti_slop::{
        config::{Origin, Policy, SCHEMA_URL},
        workspace::SourceRange,
    };
    let root = repository();
    let document = serde_json::json!({"$schema":SCHEMA_URL,"rules":{"R1":"block"},"overrides":[{"include":["**/*.ts"],"rules":{"R1":"off"}}]}).to_string();
    fs::write(root.path().join(".anti-slop.jsonc"), &document).unwrap();
    fs::write(root.path().join("a.ts"), "é😀\n").unwrap();
    fs::write(root.path().join("empty"), "").unwrap();
    git(root.path(), &["add", "a.ts", "empty"]);
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["a.ts".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(capture.stats().retained_files, 5);
    assert_eq!(capture.stats().retained_bytes, document.len() + 19);
    let ts = capture.configuration().resolve(Path::new("a.ts")).unwrap();
    let php = capture.configuration().resolve(Path::new("a.php")).unwrap();
    assert_eq!(
        capture.configuration().captured_bytes(),
        Some(document.as_bytes())
    );
    assert_eq!(ts.view().rules[0].policy, Policy::Off);
    assert!(matches!(
        ts.view().rules[0].origin,
        Origin::Override { index: 0, .. }
    ));
    assert_eq!(php.view().rules[0].policy, Policy::Block);
    assert_eq!(ts.view().source, php.view().source);
    let selected = capture.changes()[0].after.as_ref().unwrap();
    assert_ne!(selected.generation, capture.context()[0].generation);
    let mut range = SourceRange {
        path: "a.ts".into(),
        role: Role::Index,
        generation: selected.generation.clone(),
        range: 0..6,
    };
    assert_eq!(capture.slice(&range).unwrap(), "é😀".as_bytes());
    range.range = 7..7;
    assert_eq!(capture.slice(&range).unwrap(), b"");
    range.range = 0..8;
    assert!(capture.slice(&range).is_err());
    range.range = std::ops::Range { start: 3, end: 2 };
    assert!(capture.slice(&range).is_err());
    range.range = 0..1;
    range.role = Role::Context;
    assert!(capture.slice(&range).is_err());
    let empty = capture
        .changes()
        .iter()
        .find(|c| c.path == "empty")
        .unwrap()
        .after
        .as_ref()
        .unwrap();
    assert_eq!(
        capture
            .slice(&SourceRange {
                path: "empty".into(),
                role: Role::Index,
                generation: empty.generation.clone(),
                range: 0..0
            })
            .unwrap(),
        b""
    );
    fs::write(
        root.path().join(".anti-slop.jsonc"),
        document.replace("block", "advisory"),
    )
    .unwrap();
    let next = capture_staged(root.path(), &StagedRequest::default()).unwrap();
    assert_ne!(
        capture.configuration().generation(),
        next.configuration().generation()
    );
    fs::remove_file(root.path().join(".anti-slop.jsonc")).unwrap();
    assert_eq!(
        capture
            .configuration()
            .resolve(Path::new("a.php"))
            .unwrap()
            .view()
            .rules[0]
            .policy,
        Policy::Block
    );
    assert_eq!(
        capture.configuration().captured_bytes(),
        Some(document.as_bytes())
    );
}

// Catches a missing input being silently replaced or an excluded view being retained.
#[test]
fn exclusions_required_inputs_and_workspace_boundaries_are_explicit() {
    let root = repository();
    fs::write(root.path().join("a.ts"), b"selected").unwrap();
    git(root.path(), &["add", "a.ts"]);
    for (context, exclusions, expected) in [
        (
            vec![],
            vec!["folder/../a.ts".into()],
            vec![("a.ts", Role::Head), ("a.ts", Role::Index)],
        ),
        (
            vec!["staged-only.ts".into()],
            vec!["./staged-only.ts".into()],
            vec![("staged-only.ts", Role::Context)],
        ),
        (
            vec![],
            vec![".anti-slop.jsonc".into()],
            vec![(".anti-slop.jsonc", Role::Configuration)],
        ),
    ] {
        let error = capture_staged(
            root.path(),
            &StagedRequest {
                context,
                exclusions,
                ..Default::default()
            },
        )
        .unwrap_err();
        let anti_slop::workspace::CaptureError::Excluded(observed) = error else {
            panic!("{error}")
        };
        assert_eq!(
            observed
                .iter()
                .map(|i| (i.path.as_str(), i.role))
                .collect::<Vec<_>>(),
            expected
        );
    }
    for path in ["../escape", "/absolute", "", "missing"] {
        assert!(
            capture_staged(
                root.path(),
                &StagedRequest {
                    context: vec![path.into()],
                    ..Default::default()
                }
            )
            .is_err(),
            "{path}"
        );
    }
    let sub = root.path().join("sub");
    fs::create_dir(&sub).unwrap();
    assert!(capture_staged(&sub, &StagedRequest::default()).is_err());
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("secret"), b"outside").unwrap();
    use std::{
        ffi::CString,
        io::Read,
        os::fd::{AsRawFd, FromRawFd},
    };
    unsafe extern "C" {
        fn inotify_init1(flags: i32) -> i32;
        fn inotify_add_watch(fd: i32, path: *const std::ffi::c_char, mask: u32) -> i32;
    }
    // Linux IN_NONBLOCK, IN_CLOEXEC, IN_ACCESS and IN_OPEN observe actual escaped reads.
    let fd = unsafe { inotify_init1(0o4000 | 0o2000000) };
    assert!(fd >= 0);
    let mut watcher = unsafe { fs::File::from_raw_fd(fd) };
    let watched =
        CString::new(outside.path().join("secret").as_os_str().as_encoded_bytes()).unwrap();
    assert!(unsafe { inotify_add_watch(watcher.as_raw_fd(), watched.as_ptr(), 0x1 | 0x20) } >= 0);
    std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
    assert!(
        capture_staged(
            root.path(),
            &StagedRequest {
                context: vec!["escape/secret".into()],
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(
        watcher.read(&mut [0; 4096]).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "The escaped target must never be opened or read"
    );
    let config = outside.path().join("external.jsonc");
    fs::write(
        &config,
        serde_json::json!({"$schema":anti_slop::config::SCHEMA_URL,"rules":{"R1":"off"}})
            .to_string(),
    )
    .unwrap();
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            configuration: Some(config),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        capture
            .configuration()
            .resolve(Path::new("a.ts"))
            .unwrap()
            .view()
            .rules[0]
            .policy,
        anti_slop::config::Policy::Off
    );
}

#[test]
fn file_acquisition_is_bounded_before_success() {
    let root = repository();
    let huge = vec![b'x'; anti_slop::workspace::MAX_FILE_BYTES + 1];
    fs::write(root.path().join("huge"), &huge).unwrap();
    let error = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["huge".into()],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("Limit"), "{error}");
    git(root.path(), &["add", "huge"]);
    assert!(capture_staged(root.path(), &StagedRequest::default()).is_err());
}

#[test]
fn additions_deletions_literal_paths_and_empty_selection_are_distinct() {
    let root = repository();
    assert!(
        capture_staged(root.path(), &StagedRequest::default())
            .unwrap()
            .changes()
            .is_empty()
    );
    fs::write(root.path().join("untracked"), b"not selected").unwrap();
    fs::write(root.path().join("staged-only.ts"), b"worktree only").unwrap();
    assert!(
        capture_staged(root.path(), &StagedRequest::default())
            .unwrap()
            .changes()
            .is_empty()
    );
    git(root.path(), &["rm", "-q", "a.ts"]);
    for path in ["space name", "tab\tname", "line\nname", "back\\slash"] {
        fs::write(root.path().join(path), b"literal\0bytes").unwrap();
        git(root.path(), &["add", "--", path]);
    }
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["space name".into(), "./space name".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(capture.changes().len(), 5);
    assert_eq!(capture.context().len(), 1);
    let deleted = capture.changes().iter().find(|c| c.path == "a.ts").unwrap();
    assert!(deleted.after.is_none());
    assert_eq!(deleted.before.as_ref().unwrap().bytes(), b"head\n");
    for path in ["space name", "tab\tname", "line\nname", "back\\slash"] {
        let added = capture.changes().iter().find(|c| c.path == path).unwrap();
        assert!(added.before.is_none());
        assert_eq!(added.after.as_ref().unwrap().bytes(), b"literal\0bytes");
        let error = capture_staged(
            root.path(),
            &StagedRequest {
                exclusions: vec![path.into(), path.into()],
                ..Default::default()
            },
        )
        .unwrap_err();
        let anti_slop::workspace::CaptureError::Excluded(observed) = error else {
            panic!("{error}")
        };
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].path, path);
        assert_eq!(observed[0].role, Role::Index);
    }
}

#[test]
fn missing_head_unmerged_index_and_unsupported_kinds_cannot_succeed() {
    use anti_slop::workspace::CaptureError;
    let empty = TempDir::new().unwrap();
    git(empty.path(), &["init", "-q"]);
    assert!(matches!(
        capture_staged(empty.path(), &StagedRequest::default()),
        Err(CaptureError::Unavailable(_))
    ));
    let root = repository();
    let blob = git(root.path(), &["rev-parse", "HEAD:a.ts"]);
    git(root.path(), &["update-index", "--force-remove", "a.ts"]);
    let data = format!("100644 {blob} 1\ta.ts\n100644 {blob} 2\ta.ts\n100644 {blob} 3\ta.ts\n");
    use std::io::Write;
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(data.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(CaptureError::Unsupported(_))
    ));
    git(root.path(), &["reset", "-q", "HEAD"]);
    std::os::unix::fs::symlink("a.ts", root.path().join("link")).unwrap();
    git(root.path(), &["add", "link"]);
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(CaptureError::Unsupported(_))
    ));
    git(root.path(), &["reset", "-q", "HEAD"]);
    use std::os::unix::ffi::OsStringExt;
    let invalid = std::ffi::OsString::from_vec(vec![0xff, b'a']);
    fs::write(root.path().join(&invalid), b"invalid name").unwrap();
    let status = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .arg("add")
        .arg("--")
        .arg(&invalid)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(CaptureError::Unsupported(_))
    ));
}

#[test]
fn capture_preserves_git_state_and_does_not_execute_filters_or_diff_helpers() {
    let root = repository();
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    fs::write(root.path().join("a.ts"), b"working\n").unwrap();
    fs::write(
        root.path().join(".gitattributes"),
        b"*.ts filter=probe diff=probe\n",
    )
    .unwrap();
    let helper = format!("touch '{}'; cat", root.path().join("helper-ran").display());
    for key in [
        "filter.probe.clean",
        "filter.probe.smudge",
        "filter.probe.process",
        "diff.probe.command",
        "diff.probe.textconv",
        "diff.external",
        "core.fsmonitor",
    ] {
        git(root.path(), &["config", key, &helper]);
    }
    let index = fs::read(root.path().join(".git/index")).unwrap();
    let head = fs::read(root.path().join(".git/HEAD")).unwrap();
    let refs = git(root.path(), &["show-ref"]);
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["a.ts".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        capture.changes()[0].after.as_ref().unwrap().bytes(),
        b"index\n"
    );
    assert_eq!(capture.context()[0].bytes(), b"working\n");
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
    assert_eq!(fs::read(root.path().join(".git/HEAD")).unwrap(), head);
    assert_eq!(git(root.path(), &["show-ref"]), refs);
    assert_eq!(fs::read(root.path().join("a.ts")).unwrap(), b"working\n");
    assert!(!root.path().join("helper-ran").exists());
}

#[test]
fn missing_promised_object_is_not_retrieved() {
    let root = repository();
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    let blob = git(root.path(), &["rev-parse", ":a.ts"]);
    fs::remove_file(
        root.path()
            .join(".git/objects")
            .join(&blob[..2])
            .join(&blob[2..]),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    let helper = root.path().join("remote-probe");
    fs::write(
        &helper,
        format!(
            "#!/bin/sh\ntouch {}\nexit 1\n",
            shell_quote(&root.path().join("retrieved"))
        ),
    )
    .unwrap();
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
    let remote = format!("ext::{}", helper.display());
    git(root.path(), &["config", "remote.origin.url", &remote]);
    git(root.path(), &["config", "remote.origin.promisor", "true"]);
    git(
        root.path(),
        &["config", "extensions.partialClone", "origin"],
    );
    git(root.path(), &["config", "protocol.ext.allow", "always"]);
    let probe = Command::new("git")
        .arg("-C")
        .arg(root.path())
        .args(["ls-remote", "origin"])
        .output()
        .unwrap();
    assert!(!probe.status.success());
    assert!(
        root.path().join("retrieved").exists(),
        "Remote probe must actually be observable"
    );
    fs::remove_file(root.path().join("retrieved")).unwrap();
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(anti_slop::workspace::CaptureError::Unavailable(_))
    ));
    assert!(!root.path().join("retrieved").exists());
}

#[test]
fn retained_count_total_bytes_and_sparse_input_are_bounded() {
    use anti_slop::workspace::{CaptureError, MAX_FILE_BYTES, MAX_FILES};
    let root = repository();
    let sparse = fs::File::create(root.path().join("sparse")).unwrap();
    sparse.set_len(1 << 40).unwrap();
    assert!(matches!(
        capture_staged(
            root.path(),
            &StagedRequest {
                context: vec!["sparse".into()],
                ..Default::default()
            }
        ),
        Err(CaptureError::Limit(_))
    ));
    for i in 0..=MAX_FILES {
        fs::write(root.path().join(format!("empty-{i:04}")), b"").unwrap();
    }
    git(root.path(), &["add", "--", "empty-*"]);
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(CaptureError::Limit(_))
    ));
    git(root.path(), &["reset", "-q", "HEAD"]);
    let bytes = vec![b'x'; MAX_FILE_BYTES];
    for i in 0..17 {
        let name = format!("large-{i:02}");
        fs::write(root.path().join(&name), &bytes).unwrap();
        git(root.path(), &["add", &name]);
    }
    assert!(matches!(
        capture_staged(root.path(), &StagedRequest::default()),
        Err(CaptureError::Limit(_))
    ));
}

#[test]
fn relative_external_configuration_keeps_existing_resolution_semantics() {
    let root = repository();
    let external = tempfile::NamedTempFile::new_in(root.path().parent().unwrap()).unwrap();
    fs::write(
        external.path(),
        serde_json::json!({"$schema":anti_slop::config::SCHEMA_URL,"rules":{"R2":"block"}})
            .to_string(),
    )
    .unwrap();
    let relative = Path::new("..").join(external.path().file_name().unwrap());
    let capture = capture_staged(
        root.path(),
        &StagedRequest {
            configuration: Some(relative),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        capture
            .configuration()
            .resolve(Path::new("a.ts"))
            .unwrap()
            .view()
            .rules[1]
            .policy,
        anti_slop::config::Policy::Block
    );
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}

// A subprocess isolates PATH. The shim controls only the external Git timing boundary;
// inventory, blobs, mutable files and configuration are real production inputs.
fn controlled_git(kind: &str, intervention: &str) {
    use std::os::unix::fs::PermissionsExt;
    let root = repository();
    fs::write(root.path().join("a.ts"), b"initial index\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    fs::write(root.path().join("a.ts"), b"initial context\n").unwrap();
    fs::write(
        root.path().join(".anti-slop.jsonc"),
        serde_json::json!({"$schema":anti_slop::config::SCHEMA_URL,"rules":{"R1":"off"}})
            .to_string(),
    )
    .unwrap();
    if kind == "oversize" {
        fs::write(
            root.path().join("a.ts"),
            vec![b'x'; anti_slop::workspace::MAX_FILE_BYTES + 1],
        )
        .unwrap();
        git(root.path(), &["add", "a.ts"]);
        fs::write(
            root.path().join("oversized-blob"),
            git(root.path(), &["rev-parse", ":a.ts"]),
        )
        .unwrap();
    }
    fs::write(
        root.path().join("expected-head"),
        git(root.path(), &["rev-parse", "HEAD"]),
    )
    .unwrap();
    let shim = TempDir::new().unwrap();
    let script = format!(
        "#!/bin/sh\ncd {} || exit 90\nprintf '%s\\n' \"$*\" >> git-calls\n{}\nexec /usr/bin/git \"$@\"\n",
        shell_quote(root.path()),
        intervention
    );
    fs::write(shim.path().join("git"), script).unwrap();
    fs::set_permissions(shim.path().join("git"), fs::Permissions::from_mode(0o755)).unwrap();
    let start = std::time::Instant::now();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "controlled_git_child",
            "--nocapture",
        ])
        .env("PATH", format!("{}:/usr/bin:/bin", shim.path().display()))
        .env("CAPTURE_SCENARIO_ROOT", root.path())
        .env("CAPTURE_SCENARIO_KIND", kind)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{kind}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if matches!(kind, "stall" | "inventory") {
        let pid: u32 = fs::read_to_string(root.path().join("owned-pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "Owned Git must be reaped"
        );
        assert!(
            start.elapsed() < std::time::Duration::from_secs(6),
            "Bounded subprocess scenario exceeded cleanup allowance"
        );
    }
    if kind == "oversize" {
        assert!(
            !root.path().join("blob-body-read").exists(),
            "Oversized Git blob bodies must not be requested"
        );
    }
    if kind == "mutation" {
        let log = fs::read_to_string(root.path().join("git-calls")).unwrap();
        assert_eq!(
            log.lines()
                .filter(|s| s.ends_with("--verify HEAD^{commit}"))
                .count(),
            1
        );
    }
}

#[test]
#[ignore = "subprocess entrypoint for controlled external Git observations"]
fn controlled_git_child() {
    use anti_slop::workspace::CaptureError;
    let root = std::env::var_os("CAPTURE_SCENARIO_ROOT").expect("controlled fixture");
    let kind = std::env::var("CAPTURE_SCENARIO_KIND").unwrap();
    let result = capture_staged(
        Path::new(&root),
        &StagedRequest {
            context: vec!["a.ts".into()],
            ..Default::default()
        },
    );
    match kind.as_str() {
        "mutation" => {
            let capture = result.unwrap();
            assert_eq!(capture.stats().attempts, 2);
            assert_eq!(
                capture.head(),
                fs::read_to_string(Path::new(&root).join("expected-head")).unwrap()
            );
            assert_ne!(
                capture.head(),
                fs::read_to_string(Path::new(&root).join("advanced-head"))
                    .unwrap()
                    .trim()
            );
            assert_eq!(
                capture.changes()[0].after.as_ref().unwrap().bytes(),
                b"mutated index\n"
            );
            assert_eq!(capture.context()[0].bytes(), b"mutated context\n");
            assert_eq!(
                capture
                    .configuration()
                    .resolve(Path::new("a.ts"))
                    .unwrap()
                    .view()
                    .rules[0]
                    .policy,
                anti_slop::config::Policy::Block
            );
            assert_eq!(
                capture.changes()[0]
                    .after
                    .as_ref()
                    .unwrap()
                    .git_blob
                    .as_deref(),
                Some(
                    fs::read_to_string(Path::new(&root).join("expected-blob"))
                        .unwrap()
                        .trim()
                )
            );
        }
        "unstable" => assert!(matches!(result, Err(CaptureError::Unstable)), "{result:?}"),
        "stall" => assert!(matches!(result, Err(CaptureError::Timeout)), "{result:?}"),
        "oversize" => assert!(matches!(result, Err(CaptureError::Limit(_))), "{result:?}"),
        "inventory" => assert!(matches!(result, Err(CaptureError::Limit(_))), "{result:?}"),
        _ => panic!("unknown controlled scenario"),
    }
}

#[test]
fn relevant_index_configuration_and_context_mutation_retries_a_bound_generation() {
    controlled_git(
        "mutation",
        r#"
case " $* " in
  *" cat-file "*)
    if [ ! -f intervened ]; then
      touch intervened
      /usr/bin/git -c user.name=Fixture -c user.email=fixture@example.invalid commit-tree 'HEAD^{tree}' -p HEAD -m advanced > advanced-head || exit 94
      /usr/bin/git update-ref HEAD "$(cat advanced-head)" || exit 95
      printf 'mutated index\n' > a.ts
      /usr/bin/git add a.ts || exit 91
      /usr/bin/git rev-parse :a.ts > expected-blob
      printf 'mutated context\n' > a.ts
      sed -i 's/"off"/"block"/' .anti-slop.jsonc
    fi
    ;;
esac
"#,
    );
}

#[test]
fn repeatedly_changing_inventory_is_explicitly_unstable() {
    controlled_git(
        "unstable",
        r#"
case " $* " in
  *" diff-index "*)
    /usr/bin/git "$@" || exit 92
    printf 'changing\n' >> a.ts
    /usr/bin/git add a.ts || exit 93
    exit 0
    ;;
esac
"#,
    );
}

#[test]
fn stalled_git_is_killed_and_reaped() {
    controlled_git("stall", "echo $$ > owned-pid\nexec /bin/sleep 30");
}

#[test]
fn expanding_git_inventory_stops_at_its_byte_limit() {
    controlled_git(
        "inventory",
        r#"
case " $* " in
  *" diff-index "*)
    echo $$ > owned-pid
    exec /usr/bin/yes growing-inventory
    ;;
esac
"#,
    );
}

#[test]
fn executable_mode_changes_belong_to_the_captured_generation() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository();
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    let regular = capture_staged(root.path(), &StagedRequest::default()).unwrap();
    fs::set_permissions(root.path().join("a.ts"), fs::Permissions::from_mode(0o755)).unwrap();
    git(root.path(), &["add", "a.ts"]);
    let executable = capture_staged(root.path(), &StagedRequest::default()).unwrap();
    assert_ne!(
        regular.generation(),
        executable.generation(),
        "Git executable mode is a different captured input even when bytes agree"
    );
    assert_eq!(
        executable.changes()[0].before.as_ref().unwrap().git_mode,
        Some(anti_slop::workspace::GitMode::Regular)
    );
    assert_eq!(
        executable.changes()[0].after.as_ref().unwrap().git_mode,
        Some(anti_slop::workspace::GitMode::Executable)
    );
}

#[test]
fn absent_and_invalid_selected_configuration_fail_without_defaults() {
    let root = repository();
    let request = StagedRequest {
        configuration: Some("required.jsonc".into()),
        ..Default::default()
    };
    assert!(matches!(
        capture_staged(root.path(), &request),
        Err(anti_slop::workspace::CaptureError::Configuration(_))
    ));
    fs::write(root.path().join("required.jsonc"), b"{bad").unwrap();
    assert!(matches!(
        capture_staged(root.path(), &request),
        Err(anti_slop::workspace::CaptureError::Configuration(_))
    ));
    fs::write(root.path().join(".anti-slop.jsonc"), b"{bad").unwrap();
    let error = capture_staged(
        root.path(),
        &StagedRequest {
            exclusions: vec![".anti-slop.jsonc".into()],
            ..Default::default()
        },
    )
    .unwrap_err();
    let anti_slop::workspace::CaptureError::Excluded(observed) = error else {
        panic!("{error}")
    };
    assert_eq!(
        observed,
        vec![anti_slop::workspace::ObservedInput {
            path: ".anti-slop.jsonc".into(),
            role: Role::Configuration
        }]
    );
}

#[test]
fn live_growing_context_cannot_escape_acquisition_limits() {
    use std::{
        io::Write,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };
    let root = repository();
    fs::write(root.path().join("growing"), b"").unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let other = stop.clone();
    let path = root.path().join("growing");
    let (send, receive) = std::sync::mpsc::channel();
    let writer = std::thread::spawn(move || {
        let mut file = fs::OpenOptions::new().append(true).open(path).unwrap();
        let chunk = [b'x'; 8192];
        for _ in 0..128 {
            file.write_all(&chunk).unwrap();
        }
        send.send(()).unwrap();
        while !other.load(Ordering::Relaxed) {
            // Keep the fixture bounded too, while repeatedly changing the observed input.
            file.set_len(anti_slop::workspace::MAX_FILE_BYTES as u64)
                .unwrap();
            file.write_all(&chunk).unwrap();
            std::thread::yield_now();
        }
    });
    receive.recv().unwrap();
    let result = capture_staged(
        root.path(),
        &StagedRequest {
            context: vec!["growing".into()],
            ..Default::default()
        },
    );
    stop.store(true, Ordering::Relaxed);
    writer.join().unwrap();
    assert!(
        matches!(result, Err(anti_slop::workspace::CaptureError::Limit(_))),
        "{result:?}"
    );
}

/// Reproducible nonvisual library observation; timing has no CI threshold.
#[test]
#[ignore = "run explicitly with --ignored --exact capture_observations --nocapture"]
fn capture_observations() {
    let count = std::env::var("CAPTURE_OBSERVATION_FILES")
        .unwrap_or_else(|_| "1".into())
        .parse::<usize>()
        .unwrap();
    assert!(matches!(count, 1 | 5));
    let root = repository();
    for n in 0..count {
        fs::write(
            root.path().join(format!("input-{n}.ts")),
            (0..20)
                .map(|line| format!("before-{n}-{line}\n"))
                .collect::<String>(),
        )
        .unwrap();
    }
    fs::write(
        root.path().join("consumer.ts"),
        b"// explicit consumer context\nimport './input-0';\n",
    )
    .unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "scenario"]);
    for n in 0..count {
        fs::write(
            root.path().join(format!("input-{n}.ts")),
            (0..20)
                .map(|line| format!("after-{n}-{line}\n"))
                .collect::<String>(),
        )
        .unwrap();
    }
    git(root.path(), &["add", "."]);
    let changed_lines: usize = git(root.path(), &["diff", "--cached", "--numstat"])
        .lines()
        .map(|line| {
            line.split('\t')
                .take(2)
                .map(|n| n.parse::<usize>().unwrap())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(changed_lines, 40 * count);
    let request = StagedRequest {
        context: vec!["consumer.ts".into()],
        ..Default::default()
    };
    let marker = std::env::var_os("CAPTURE_OBSERVATION_MARKER");
    if let Some(marker) = &marker {
        fs::write(marker, "capture").unwrap();
    }
    let mut micros = Vec::new();
    let mut retained = None;
    let mut processes = 0;
    for _ in 0..31 {
        let start = std::time::Instant::now();
        let capture = capture_staged(root.path(), &request).unwrap();
        micros.push(start.elapsed().as_micros() as u64);
        assert_eq!(capture.changes().len(), count);
        assert_eq!(
            capture.context()[0].bytes(),
            b"// explicit consumer context\nimport './input-0';\n"
        );
        retained = Some((
            capture.stats().retained_files,
            capture.stats().retained_bytes,
        ));
        processes = capture.stats().git_processes;
    }
    if let Some(marker) = &marker {
        fs::write(marker, "complete").unwrap();
    }
    let first = micros.remove(0);
    let raw = micros.clone();
    micros.sort_unstable();
    println!(
        "OBSERVATION {}",
        serde_json::json!({"selectedFiles": count,"changedLines": changed_lines,"contextFiles": 1,"firstCaptureMicros": first,"warmCaptureMicros":raw,"warmP50Micros":micros[14],"warmP95Micros":micros[28],"retainedInputs":retained.unwrap().0,"retainedBytes":retained.unwrap().1,"gitProcessesPerCapture":processes,"gitConcurrency":1,"cache":"first call in fresh test executable with OS caches warmed by fixture creation; new Git processes on every call; subsequent calls share caller process only; no application cache"})
    );
}

#[test]
fn oversized_git_blob_is_rejected_before_body_acquisition() {
    controlled_git(
        "oversize",
        r#"
case " $* " in
  *" cat-file blob "*)
    last=""
    for argument in "$@"; do last="$argument"; done
    if [ "$last" = "$(cat oversized-blob)" ]; then touch blob-body-read; fi
    ;;
esac
"#,
    );
}
