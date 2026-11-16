use anti_slop::workspace::{CaptureRequest, Role, Selection, capture as capture_sources};
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
    let capture = capture_sources(
        root.path(),
        &CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(Selection::Staged)
        },
    )
    .expect("staged capture must return retained HEAD/index and independent context");
    assert_eq!(
        capture.resolved_commit().unwrap(),
        git(root.path(), &["rev-parse", "HEAD"])
    );
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
    let capture = capture_sources(
        root.path(),
        &CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(Selection::Staged)
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
    let next = capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)).unwrap();
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
        let error = capture_sources(
            root.path(),
            &CaptureRequest {
                context,
                exclusions,
                ..CaptureRequest::new(Selection::Staged)
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
            capture_sources(
                root.path(),
                &CaptureRequest {
                    context: vec![path.into()],
                    ..CaptureRequest::new(Selection::Staged)
                }
            )
            .is_err(),
            "{path}"
        );
    }
    let sub = root.path().join("sub");
    fs::create_dir(&sub).unwrap();
    assert!(capture_sources(&sub, &CaptureRequest::new(Selection::Staged)).is_err());
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
        capture_sources(
            root.path(),
            &CaptureRequest {
                context: vec!["escape/secret".into()],
                ..CaptureRequest::new(Selection::Staged)
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
    let capture = capture_sources(
        root.path(),
        &CaptureRequest {
            configuration: Some(config),
            ..CaptureRequest::new(Selection::Staged)
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
    let error = capture_sources(
        root.path(),
        &CaptureRequest {
            context: vec!["huge".into()],
            ..CaptureRequest::new(Selection::Staged)
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("Limit"), "{error}");
    git(root.path(), &["add", "huge"]);
    assert!(capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)).is_err());
}

#[test]
fn additions_deletions_literal_paths_and_empty_selection_are_distinct() {
    let root = repository();
    assert!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged))
            .unwrap()
            .changes()
            .is_empty()
    );
    fs::write(root.path().join("untracked"), b"not selected").unwrap();
    fs::write(root.path().join("staged-only.ts"), b"worktree only").unwrap();
    assert!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged))
            .unwrap()
            .changes()
            .is_empty()
    );
    git(root.path(), &["rm", "-q", "a.ts"]);
    for path in ["space name", "tab\tname", "line\nname", "back\\slash"] {
        fs::write(root.path().join(path), b"literal\0bytes").unwrap();
        git(root.path(), &["add", "--", path]);
    }
    let capture = capture_sources(
        root.path(),
        &CaptureRequest {
            context: vec!["space name".into(), "./space name".into()],
            ..CaptureRequest::new(Selection::Staged)
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
        let error = capture_sources(
            root.path(),
            &CaptureRequest {
                exclusions: vec![path.into(), path.into()],
                ..CaptureRequest::new(Selection::Staged)
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
        capture_sources(empty.path(), &CaptureRequest::new(Selection::Staged)),
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
    for selection in [
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        assert!(matches!(
            capture_sources(root.path(), &CaptureRequest::new(selection)),
            Err(CaptureError::Unsupported(_))
        ));
    }
    assert!(matches!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)),
        Err(CaptureError::Unsupported(_))
    ));
    git(root.path(), &["reset", "-q", "HEAD"]);
    std::os::unix::fs::symlink("a.ts", root.path().join("link")).unwrap();
    git(root.path(), &["add", "link"]);
    assert!(matches!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)),
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
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)),
        Err(CaptureError::Unsupported(_))
    ));
}

#[test]
fn capture_preserves_git_state_and_does_not_execute_filters_or_diff_helpers() {
    for location in [".git/info/attributes", ".gitattributes", "external"] {
        let root = repository();
        fs::write(root.path().join("a.ts"), b"index\n").unwrap();
        git(root.path(), &["add", "a.ts"]);
        fs::write(root.path().join("a.ts"), b"working\n").unwrap();
        let external = TempDir::new().unwrap();
        let attribute_file = if location == "external" {
            external.path().join("attributes")
        } else {
            root.path().join(location)
        };
        fs::write(&attribute_file, b"*.ts filter=probe diff=probe\n").unwrap();
        if location == "external" {
            git(
                root.path(),
                &[
                    "config",
                    "core.attributesFile",
                    attribute_file.to_str().unwrap(),
                ],
            );
        }
        for key in [
            "filter.probe.clean",
            "filter.probe.smudge",
            "filter.probe.process",
            "diff.probe.command",
            "diff.probe.textconv",
            "diff.external",
            "core.fsmonitor",
        ] {
            let helper = format!(
                "touch '{}'; cat",
                root.path().join(format!("helper-ran-{key}")).display()
            );
            git(root.path(), &["config", key, &helper]);
        }
        let index = fs::read(root.path().join(".git/index")).unwrap();
        let head = fs::read(root.path().join(".git/HEAD")).unwrap();
        let refs = git(root.path(), &["show-ref"]);
        for selection in [
            Selection::Staged,
            Selection::Unstaged,
            Selection::Local,
            Selection::Base("HEAD".into()),
        ] {
            let markers = || {
                fs::read_dir(root.path())
                    .unwrap()
                    .map(|p| p.unwrap().file_name())
                    .filter(|p| p.to_string_lossy().starts_with("helper-ran-"))
                    .collect::<Vec<_>>()
            };
            assert!(
                markers().is_empty(),
                "Fixture ran helpers before {selection:?}: {:?}",
                markers()
            );
            let result = capture_sources(
                root.path(),
                &CaptureRequest {
                    context: vec!["a.ts".into()],
                    ..CaptureRequest::new(selection.clone())
                },
            );
            assert!(
                markers().is_empty(),
                "{selection:?} executed project helpers: {:?}",
                markers()
            );
            let captured = result.unwrap();
            let selected = captured
                .changes()
                .iter()
                .find(|c| c.path == "a.ts")
                .unwrap()
                .after
                .as_ref()
                .unwrap();
            assert_eq!(
                selected.bytes(),
                if selection == Selection::Staged {
                    b"index\n".as_slice()
                } else {
                    b"working\n".as_slice()
                }
            );
            assert_eq!(captured.context()[0].bytes(), b"working\n");
            assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
            assert_eq!(fs::read(root.path().join(".git/HEAD")).unwrap(), head);
            assert_eq!(git(root.path(), &["show-ref"]), refs);
            assert_eq!(fs::read(root.path().join("a.ts")).unwrap(), b"working\n");
        }
    }
}

#[test]
fn missing_promised_object_is_not_retrieved() {
    for selection in [
        Selection::Staged,
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        let root = repository();
        fs::write(root.path().join("a.ts"), b"index\n").unwrap();
        git(root.path(), &["add", "a.ts"]);
        fs::write(root.path().join("a.ts"), b"working\n").unwrap();
        let blob = git(
            root.path(),
            &[
                "rev-parse",
                if matches!(selection, Selection::Local | Selection::Base(_)) {
                    "HEAD:a.ts"
                } else {
                    ":a.ts"
                },
            ],
        );
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
            capture_sources(root.path(), &CaptureRequest::new(selection.clone())),
            Err(anti_slop::workspace::CaptureError::Unavailable(_))
        ));
        assert!(!root.path().join("retrieved").exists());
    }
}

#[test]
fn retained_count_total_bytes_and_sparse_input_are_bounded() {
    use anti_slop::workspace::{CaptureError, MAX_FILE_BYTES, MAX_FILES};
    let root = repository();
    let sparse = fs::File::create(root.path().join("sparse")).unwrap();
    sparse.set_len(1 << 40).unwrap();
    assert!(matches!(
        capture_sources(
            root.path(),
            &CaptureRequest {
                context: vec!["sparse".into()],
                ..CaptureRequest::new(Selection::Staged)
            }
        ),
        Err(CaptureError::Limit(_))
    ));
    for i in 0..=MAX_FILES {
        fs::write(root.path().join(format!("empty-{i:04}")), b"").unwrap();
    }
    git(root.path(), &["add", "--", "empty-*"]);
    assert!(matches!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)),
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
        capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)),
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
    let capture = capture_sources(
        root.path(),
        &CaptureRequest {
            configuration: Some(relative),
            ..CaptureRequest::new(Selection::Staged)
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
    fs::write(root.path().join(".git/info/exclude"), b"git-calls\nintervened\nenumerated\nexpected-*\nadvanced-head\nowned-pid\ndescendant-pid\noversized-blob\n").unwrap();
    if kind == "base-pin" {
        git(root.path(), &["branch", "moving"]);
    }
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
        !String::from_utf8_lossy(&output.stderr)
            .contains("OBSERVED_GROUP_SIGNAL_AFTER_LEADER_REAP"),
        "{kind}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "{kind}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if matches!(
        kind,
        "stall" | "inventory" | "early-exit" | "untracked-inventory"
    ) {
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
                .filter(|s| s.ends_with("--verify --end-of-options HEAD^{commit}"))
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
    if kind == "early-exit" {
        unsafe extern "C" {
            fn prctl(option: i32, ...) -> i32;
        }
        // Isolated test subprocess adopts the stalled descendant so the fixture can reap it.
        assert_eq!(unsafe { prctl(36, 1u64, 0u64, 0u64, 0u64) }, 0);
    }
    let selection = match kind.as_str() {
        "working-mutation" | "untracked-added" | "untracked-inventory" => Selection::Local,
        "working-unstable" => Selection::Unstaged,
        "base-pin" => Selection::Base("moving".into()),
        _ => Selection::Staged,
    };
    let result = capture_sources(
        Path::new(&root),
        &CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(selection)
        },
    );
    match kind.as_str() {
        "mutation" => {
            let capture = result.unwrap();
            assert_eq!(capture.stats().attempts, 2);
            assert_eq!(
                capture.resolved_commit().unwrap(),
                fs::read_to_string(Path::new(&root).join("expected-head")).unwrap()
            );
            assert_ne!(
                capture.resolved_commit().unwrap(),
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
        "working-mutation" => {
            let captured = result.unwrap();
            assert_eq!(captured.stats().attempts, 2);
            let source = captured
                .changes()
                .iter()
                .find(|c| c.path == "a.ts")
                .unwrap();
            assert_eq!(source.before.as_ref().unwrap().bytes(), b"head\n");
            assert_eq!(source.after.as_ref().unwrap().role, Role::Working);
            assert!(source.after.as_ref().unwrap().git_blob.is_none());
            assert_eq!(source.after.as_ref().unwrap().bytes(), b"mutated working\n");
            assert_eq!(captured.context()[0].bytes(), b"mutated working\n");
            assert_ne!(
                source.after.as_ref().unwrap().generation,
                captured.context()[0].generation
            );
        }
        "untracked-added" => {
            let captured = result.unwrap();
            assert_eq!(captured.stats().attempts, 2);
            let added = captured
                .changes()
                .iter()
                .find(|c| c.path == "created.ts")
                .unwrap();
            assert!(added.before.is_none());
            assert_eq!(
                added.after.as_ref().unwrap().bytes(),
                b"created during acquisition\n"
            );
        }
        "base-pin" => {
            let captured = result.unwrap();
            assert_eq!(
                captured.resolved_commit().unwrap(),
                fs::read_to_string(Path::new(&root).join("expected-head")).unwrap()
            );
            let before = captured
                .changes()
                .iter()
                .find(|c| c.path == "a.ts")
                .unwrap()
                .before
                .as_ref()
                .unwrap();
            assert_eq!(before.role, Role::Base);
            assert_eq!(before.bytes(), b"head\n");
            let calls = fs::read_to_string(Path::new(&root).join("git-calls")).unwrap();
            assert_eq!(
                calls
                    .lines()
                    .filter(|s| s.ends_with("--verify --end-of-options moving^{commit}"))
                    .count(),
                1
            );
        }
        "working-unstable" => assert!(matches!(result, Err(CaptureError::Unstable)), "{result:?}"),
        "untracked-inventory" => {
            assert!(matches!(result, Err(CaptureError::Limit(_))), "{result:?}")
        }
        "unstable" => assert!(matches!(result, Err(CaptureError::Unstable)), "{result:?}"),
        "stall" => assert!(matches!(result, Err(CaptureError::Timeout)), "{result:?}"),
        "early-exit" => {
            assert!(matches!(result, Err(CaptureError::Timeout)), "{result:?}");
            let pid: i32 = fs::read_to_string(Path::new(&root).join("descendant-pid"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            unsafe extern "C" {
                fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
            }
            let start = std::time::Instant::now();
            let mut status = 0;
            loop {
                let waited = unsafe { waitpid(pid, &mut status, 1) };
                if waited == pid {
                    break;
                }
                assert_eq!(waited, 0, "Failed to observe owned descendant exit");
                assert!(
                    start.elapsed() < std::time::Duration::from_secs(2),
                    "Owned descendant remained alive"
                );
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert_eq!(
                status & 0x7f,
                9,
                "Owned descendant must be killed, not merely abandoned"
            );
        }
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
fn working_modes_capture_owner_execute_removal_with_group_execute_remaining() {
    use anti_slop::workspace::GitMode;
    use std::os::unix::fs::PermissionsExt;
    let root = repository();
    let path = root.path().join("a.ts");
    git(root.path(), &["config", "core.filemode", "true"]);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o744)).unwrap();
    git(root.path(), &["add", "a.ts"]);
    git(root.path(), &["commit", "-qm", "owner executable"]);
    assert!(git(root.path(), &["ls-files", "--stage", "a.ts"]).starts_with("100755 "));
    for (selection, before_role) in [
        (Selection::Unstaged, Role::Index),
        (Selection::Local, Role::Head),
        (Selection::Base("HEAD".into()), Role::Base),
    ] {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o744)).unwrap();
        let request = CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(selection)
        };
        let executable = capture_sources(root.path(), &request).unwrap();
        assert!(executable.changes().is_empty());
        assert_eq!(executable.context()[0].git_mode, Some(GitMode::Executable));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o654)).unwrap();
        assert_eq!(
            git(
                root.path(),
                &["diff", "--no-ext-diff", "--no-textconv", "--summary"]
            ),
            "mode change 100755 => 100644 a.ts"
        );
        let captured = capture_sources(root.path(), &request).unwrap();
        assert_eq!(captured.selection(), &request.selection);
        assert_eq!(captured.stats().attempts, 1);
        assert_eq!(
            captured
                .changes()
                .iter()
                .map(|c| c.path.as_str())
                .collect::<Vec<_>>(),
            ["a.ts"],
            "{:?} must retain the real Git executable-mode removal",
            request.selection
        );
        let change = &captured.changes()[0];
        let before = change.before.as_ref().unwrap();
        let after = change.after.as_ref().unwrap();
        assert_eq!(
            (before.path.as_str(), before.role, before.git_mode),
            ("a.ts", before_role, Some(GitMode::Executable))
        );
        assert_eq!(
            (after.path.as_str(), after.role, after.git_mode),
            ("a.ts", Role::Working, Some(GitMode::Regular))
        );
        assert_eq!(before.bytes(), b"head\n");
        assert_eq!(after.bytes(), b"head\n");
        assert_eq!(captured.context().len(), 1);
        let context = &captured.context()[0];
        assert_eq!(
            (context.path.as_str(), context.role, context.git_mode),
            ("a.ts", Role::Context, Some(GitMode::Regular))
        );
        assert_eq!(context.bytes(), after.bytes());
        assert_ne!(context.generation, after.generation);
        assert_ne!(context.generation, executable.context()[0].generation);
        assert_ne!(captured.generation(), executable.generation());
        let repeated = capture_sources(root.path(), &request).unwrap();
        assert_eq!(captured.generation(), repeated.generation());
        assert_eq!(
            after.generation,
            repeated.changes()[0].after.as_ref().unwrap().generation
        );
        assert_eq!(context.generation, repeated.context()[0].generation);
    }
}

#[test]
fn working_modes_ignore_group_or_other_execute_without_owner_execute() {
    use anti_slop::workspace::GitMode;
    use std::os::unix::fs::PermissionsExt;
    let root = repository();
    let path = root.path().join("a.ts");
    git(root.path(), &["config", "core.filemode", "true"]);
    assert!(git(root.path(), &["ls-files", "--stage", "a.ts"]).starts_with("100644 "));
    for selection in [
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let request = CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(selection)
        };
        let regular = capture_sources(root.path(), &request).unwrap();
        assert!(regular.changes().is_empty());
        for mode in [0o654, 0o645] {
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
            assert!(
                git(
                    root.path(),
                    &["diff", "--no-ext-diff", "--no-textconv", "--raw"]
                )
                .is_empty()
            );
            let captured = capture_sources(root.path(), &request).unwrap();
            assert_eq!(captured.selection(), &request.selection);
            assert_eq!(captured.stats().attempts, 1);
            assert!(
                captured.changes().is_empty(),
                "{:?} must ignore non-owner execute permission {mode:o}: {:?}",
                request.selection,
                captured.changes()
            );
            assert_eq!(captured.context().len(), 1);
            let context = &captured.context()[0];
            assert_eq!(
                (context.path.as_str(), context.role, context.git_mode),
                ("a.ts", Role::Context, Some(GitMode::Regular))
            );
            assert_eq!(context.bytes(), b"head\n");
            assert_eq!(context.generation, regular.context()[0].generation);
            assert_eq!(captured.generation(), regular.generation());
        }
    }
}

#[test]
fn executable_mode_changes_belong_to_the_captured_generation() {
    use std::os::unix::fs::PermissionsExt;
    let root = repository();
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    let regular = capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)).unwrap();
    fs::set_permissions(root.path().join("a.ts"), fs::Permissions::from_mode(0o755)).unwrap();
    git(root.path(), &["add", "a.ts"]);
    let executable = capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)).unwrap();
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
    let request = CaptureRequest {
        configuration: Some("required.jsonc".into()),
        ..CaptureRequest::new(Selection::Staged)
    };
    assert!(matches!(
        capture_sources(root.path(), &request),
        Err(anti_slop::workspace::CaptureError::Configuration(_))
    ));
    fs::write(root.path().join("required.jsonc"), b"{bad").unwrap();
    assert!(matches!(
        capture_sources(root.path(), &request),
        Err(anti_slop::workspace::CaptureError::Configuration(_))
    ));
    fs::write(root.path().join(".anti-slop.jsonc"), b"{bad").unwrap();
    let error = capture_sources(
        root.path(),
        &CaptureRequest {
            exclusions: vec![".anti-slop.jsonc".into()],
            ..CaptureRequest::new(Selection::Staged)
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
        for _ in 0..129 {
            file.write_all(&chunk).unwrap();
        }
        send.send(()).unwrap();
        while !other.load(Ordering::Relaxed) {
            // Keep the fixture bounded too, while repeatedly changing the observed input.
            file.set_len((anti_slop::workspace::MAX_FILE_BYTES + chunk.len()) as u64)
                .unwrap();
            file.write_all(&chunk).unwrap();
            std::thread::yield_now();
        }
    });
    receive.recv().unwrap();
    let result = capture_sources(
        root.path(),
        &CaptureRequest {
            context: vec!["growing".into()],
            ..CaptureRequest::new(Selection::Staged)
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
    let selector_name =
        std::env::var("CAPTURE_OBSERVATION_SELECTION").unwrap_or_else(|_| "staged".into());
    let selection = match selector_name.as_str() {
        "staged" => Selection::Staged,
        "unstaged" => Selection::Unstaged,
        "local" => Selection::Local,
        "base" => Selection::Base("HEAD".into()),
        _ => panic!("unknown observation selector"),
    };
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
    if selection != Selection::Unstaged {
        git(root.path(), &["add", "."]);
    }
    let changed_lines: usize = git(
        root.path(),
        if selection == Selection::Unstaged {
            &["diff", "--numstat"]
        } else {
            &["diff", "--cached", "--numstat"]
        },
    )
    .lines()
    .map(|line| {
        line.split('\t')
            .take(2)
            .map(|n| n.parse::<usize>().unwrap())
            .sum::<usize>()
    })
    .sum();
    assert_eq!(changed_lines, 40 * count);
    let request = CaptureRequest {
        context: vec!["consumer.ts".into()],
        ..CaptureRequest::new(selection)
    };
    let marker = std::env::var_os("CAPTURE_OBSERVATION_MARKER");
    if let Some(marker) = &marker {
        fs::write(marker, "capture").unwrap();
    }
    let mut micros = Vec::new();
    let mut retained = None;
    let mut processes = 0;
    let mut acquisition = (0, 0);
    for _ in 0..31 {
        let start = std::time::Instant::now();
        let capture = capture_sources(root.path(), &request).unwrap();
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
        acquisition = (
            capture.stats().acquired_files,
            capture.stats().acquired_bytes,
        );
    }
    if let Some(marker) = &marker {
        fs::write(marker, "complete").unwrap();
    }
    let first = micros.remove(0);
    let raw = micros.clone();
    micros.sort_unstable();
    println!(
        "OBSERVATION {}",
        serde_json::json!({"selection":selector_name,"acquiredInputs":acquisition.0,"acquiredBytes":acquisition.1,"selectedFiles": count,"changedLines": changed_lines,"contextFiles": 1,"firstCaptureMicros": first,"warmCaptureMicros":raw,"warmP50Micros":micros[14],"warmP95Micros":micros[28],"retainedInputs":retained.unwrap().0,"retainedBytes":retained.unwrap().1,"gitProcessesPerCapture":processes,"gitConcurrency":1,"cache":"first call in fresh test executable with OS caches warmed by fixture creation; new Git processes on every call; subsequent calls share caller process only; no application cache"})
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

// Observes the actual caller path's syscalls while the existing workflow assertions stay real.
#[test]
fn owned_group_signals_reserve_the_leader_until_cleanup_finishes() {
    let observer = TempDir::new().unwrap();
    let library = observer.path().join("observe-kill.so");
    let build = Command::new("cc")
        .args(["-shared", "-fPIC", "-O2"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/process/observe-kill.c"))
        .args(["-o"])
        .arg(&library)
        .arg("-ldl")
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    for scenario in [
        "staged_bytes_and_working_context_remain_independent",
        "stalled_git_is_killed_and_reaped",
        "expanding_git_inventory_stops_at_its_byte_limit",
        "early_exited_git_leader_cleans_up_stalled_descendant_pipe",
    ] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([scenario, "--exact", "--nocapture", "--test-threads=1"])
            .env("LD_PRELOAD", &library)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{scenario}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr)
                .contains("OBSERVED_GROUP_SIGNAL_AFTER_LEADER_REAP"),
            "{scenario}: Group signaling after leader PID reservation was lost: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn early_exited_git_leader_cleans_up_stalled_descendant_pipe() {
    controlled_git(
        "early-exit",
        "echo $$ > owned-pid\n/bin/sleep 30 &\necho $! > descendant-pid\nexit 0",
    );
}

// Distinct explicit base, merge-base, HEAD, index and working bytes catch comparison substitution.
#[test]
fn four_comparisons_select_their_exact_sides_and_independent_context() {
    let root = repository();
    git(root.path(), &["branch", "common"]);
    git(root.path(), &["checkout", "-qb", "release"]);
    fs::write(root.path().join("a.ts"), b"base\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    git(root.path(), &["commit", "-qm", "base"]);
    let base = git(root.path(), &["rev-parse", "HEAD"]);
    git(root.path(), &["checkout", "-q", "common"]);
    fs::write(root.path().join("a.ts"), b"new head\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    git(root.path(), &["commit", "-qm", "head"]);
    let head = git(root.path(), &["rev-parse", "HEAD"]);
    assert_ne!(git(root.path(), &["merge-base", "HEAD", "release"]), base);
    fs::write(root.path().join("a.ts"), b"index\n").unwrap();
    fs::write(root.path().join("staged-only.ts"), b"staged-only\n").unwrap();
    git(root.path(), &["add", "."]);
    fs::write(root.path().join("a.ts"), b"working\n").unwrap();
    fs::write(root.path().join("untracked.ts"), b"untracked\n").unwrap();
    fs::write(root.path().join(".gitignore"), b"ignored.ts\n").unwrap();
    git(root.path(), &["add", ".gitignore"]);
    fs::write(root.path().join("ignored.ts"), b"ignored\n").unwrap();
    for (selection, before, after, expected_paths) in [
        (
            Selection::Staged,
            b"new head\n".as_slice(),
            b"index\n".as_slice(),
            vec![".gitignore", "a.ts", "staged-only.ts"],
        ),
        (
            Selection::Unstaged,
            b"index\n".as_slice(),
            b"working\n".as_slice(),
            vec!["a.ts", "untracked.ts"],
        ),
        (
            Selection::Local,
            b"new head\n".as_slice(),
            b"working\n".as_slice(),
            vec![".gitignore", "a.ts", "staged-only.ts", "untracked.ts"],
        ),
        (
            Selection::Base("release".into()),
            b"base\n".as_slice(),
            b"working\n".as_slice(),
            vec![".gitignore", "a.ts", "staged-only.ts", "untracked.ts"],
        ),
    ] {
        let request = CaptureRequest {
            context: vec!["a.ts".into()],
            ..CaptureRequest::new(selection.clone())
        };
        let captured = capture_sources(root.path(), &request)
            .expect("every explicit comparison must capture its intended sources");
        assert_eq!(
            captured
                .changes()
                .iter()
                .map(|c| c.path.as_str())
                .collect::<Vec<_>>(),
            expected_paths,
            "{selection:?}"
        );
        let change = captured
            .changes()
            .iter()
            .find(|c| c.path == "a.ts")
            .unwrap();
        assert_eq!(change.before.as_ref().unwrap().bytes(), before);
        assert_eq!(change.after.as_ref().unwrap().bytes(), after);
        assert_eq!(captured.context()[0].bytes(), b"working\n");
        assert_ne!(
            change.after.as_ref().unwrap().generation,
            captured.context()[0].generation
        );
        assert_eq!(captured.selection(), &selection);
        let before_role = match selection {
            Selection::Staged | Selection::Local => Role::Head,
            Selection::Unstaged => Role::Index,
            Selection::Base(_) => Role::Base,
        };
        assert_eq!(change.before.as_ref().unwrap().role, before_role);
        assert_eq!(
            change.after.as_ref().unwrap().role,
            if selection == Selection::Staged {
                Role::Index
            } else {
                Role::Working
            }
        );
        assert!(change.before.as_ref().unwrap().git_blob.is_some());
        assert_eq!(
            change.after.as_ref().unwrap().git_blob.is_some(),
            selection == Selection::Staged
        );
        assert_eq!(captured.context()[0].role, Role::Context);
        let expected_commit = match selection {
            Selection::Unstaged => None,
            Selection::Base(_) => Some(base.as_str()),
            _ => Some(head.as_str()),
        };
        assert_eq!(captured.resolved_commit(), expected_commit);
        println!(
            "DEMO selection={selection:?} commit={:?} paths={:?} a.before={:?}/{:?} a.after={:?}/{:?} context={:?}",
            captured.resolved_commit(),
            captured
                .changes()
                .iter()
                .map(|c| c.path.as_str())
                .collect::<Vec<_>>(),
            change.before.as_ref().unwrap().role,
            std::str::from_utf8(before).unwrap(),
            change.after.as_ref().unwrap().role,
            std::str::from_utf8(after).unwrap(),
            std::str::from_utf8(captured.context()[0].bytes()).unwrap()
        );
    }
}

#[test]
fn exact_content_renames_keep_both_paths_and_reject_either_exclusion() {
    for selection in [
        Selection::Staged,
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        let root = repository();
        let destination = "moved\tname\n.ts";
        fs::rename(root.path().join("a.ts"), root.path().join(destination)).unwrap();
        if selection == Selection::Staged {
            git(root.path(), &["add", "-A"]);
        }
        let captured =
            capture_sources(root.path(), &CaptureRequest::new(selection.clone())).unwrap();
        assert_eq!(
            captured.changes().len(),
            1,
            "Unique identical deletion/addition must be attributed once: {selection:?}"
        );
        let renamed = &captured.changes()[0];
        assert_eq!(renamed.path, destination);
        assert_eq!(renamed.before.as_ref().unwrap().path, "a.ts");
        assert_eq!(renamed.after.as_ref().unwrap().path, destination);
        assert_eq!(renamed.before.as_ref().unwrap().bytes(), b"head\n");
        assert_eq!(renamed.after.as_ref().unwrap().bytes(), b"head\n");
        for excluded in ["a.ts", destination] {
            let error = capture_sources(
                root.path(),
                &CaptureRequest {
                    exclusions: vec![excluded.into()],
                    ..CaptureRequest::new(selection.clone())
                },
            )
            .unwrap_err();
            let anti_slop::workspace::CaptureError::Excluded(observed) = error else {
                panic!("{error}")
            };
            assert!(observed.iter().any(|i| i.path == excluded));
        }
    }
}

#[test]
fn identical_ambiguous_rename_groups_remain_unpaired_evidence() {
    let root = repository();
    fs::write(root.path().join("second.ts"), b"head\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "identical sources"]);
    fs::rename(root.path().join("a.ts"), root.path().join("x.ts")).unwrap();
    fs::rename(root.path().join("second.ts"), root.path().join("y.ts")).unwrap();
    git(root.path(), &["add", "-A"]);
    for selection in [Selection::Staged, Selection::Local] {
        let captured = capture_sources(root.path(), &CaptureRequest::new(selection)).unwrap();
        assert_eq!(
            captured
                .changes()
                .iter()
                .map(|c| c.path.as_str())
                .collect::<Vec<_>>(),
            ["a.ts", "second.ts", "x.ts", "y.ts"]
        );
        assert!(
            captured
                .changes()
                .iter()
                .all(|c| c.before.is_none() != c.after.is_none())
        );
        for source in captured
            .changes()
            .iter()
            .flat_map(|c| c.before.iter().chain(c.after.iter()))
        {
            assert_eq!(source.bytes(), b"head\n");
        }
    }
}

#[test]
fn working_selections_reject_index_flags_that_hide_mutable_changes() {
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        let root = repository();
        git(root.path(), &["update-index", flag, "a.ts"]);
        fs::write(root.path().join("a.ts"), b"hidden working change\n").unwrap();
        for selection in [
            Selection::Unstaged,
            Selection::Local,
            Selection::Base("HEAD".into()),
        ] {
            let result = capture_sources(root.path(), &CaptureRequest::new(selection.clone()));
            assert!(
                matches!(
                    result,
                    Err(anti_slop::workspace::CaptureError::Unsupported(_))
                ),
                "{flag} {selection:?}: {result:?}"
            );
        }
    }
}

#[test]
fn intent_to_add_is_absent_from_staged_and_is_a_working_addition() {
    let root = repository();
    fs::write(root.path().join("intent.ts"), b"intent working bytes\n").unwrap();
    git(root.path(), &["add", "-N", "intent.ts"]);
    let staged = capture_sources(root.path(), &CaptureRequest::new(Selection::Staged)).unwrap();
    assert!(
        staged.changes().is_empty(),
        "Intent is not a staged empty-file addition: {staged:?}"
    );
    for selection in [
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        let captured =
            capture_sources(root.path(), &CaptureRequest::new(selection.clone())).unwrap();
        assert_eq!(captured.changes().len(), 1, "{selection:?}");
        let change = &captured.changes()[0];
        assert_eq!(change.path, "intent.ts");
        assert!(
            change.before.is_none(),
            "Intent must not fabricate prior empty content"
        );
        assert_eq!(
            change.after.as_ref().unwrap().bytes(),
            b"intent working bytes\n"
        );
        assert_eq!(change.after.as_ref().unwrap().role, Role::Working);
    }
}

#[test]
fn cancellation_empty_comparisons_and_required_head_are_distinct() {
    use anti_slop::workspace::CaptureError;
    let root = repository();
    for selection in [
        Selection::Staged,
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        assert!(
            capture_sources(root.path(), &CaptureRequest::new(selection))
                .unwrap()
                .changes()
                .is_empty()
        );
    }
    fs::write(root.path().join("a.ts"), b"index only\n").unwrap();
    git(root.path(), &["add", "a.ts"]);
    fs::write(root.path().join("a.ts"), b"head\n").unwrap();
    for selection in [Selection::Local, Selection::Base("HEAD".into())] {
        assert!(
            capture_sources(root.path(), &CaptureRequest::new(selection))
                .unwrap()
                .changes()
                .is_empty(),
            "An index change canceled by actual working bytes is not a local change"
        );
    }
    let unstaged = capture_sources(root.path(), &CaptureRequest::new(Selection::Unstaged)).unwrap();
    assert_eq!(unstaged.changes().len(), 1);
    assert_eq!(
        unstaged.changes()[0].before.as_ref().unwrap().bytes(),
        b"index only\n"
    );
    assert_eq!(
        unstaged.changes()[0].after.as_ref().unwrap().bytes(),
        b"head\n"
    );
    for reference in ["", "does-not-exist", "--help", "bad\0ref"] {
        assert!(
            capture_sources(
                root.path(),
                &CaptureRequest::new(Selection::Base(reference.into()))
            )
            .is_err()
        );
    }
    git(root.path(), &["branch", "saved"]);
    git(root.path(), &["symbolic-ref", "HEAD", "refs/heads/unborn"]);
    for selection in [Selection::Staged, Selection::Local] {
        assert!(matches!(
            capture_sources(root.path(), &CaptureRequest::new(selection)),
            Err(CaptureError::Unavailable(_))
        ));
    }
    assert_eq!(
        capture_sources(root.path(), &CaptureRequest::new(Selection::Unstaged))
            .unwrap()
            .resolved_commit(),
        None
    );
    assert!(
        capture_sources(
            root.path(),
            &CaptureRequest::new(Selection::Base("saved".into()))
        )
        .unwrap()
        .changes()
        .is_empty()
    );
    let unborn = TempDir::new().unwrap();
    git(unborn.path(), &["init", "-q"]);
    fs::write(unborn.path().join("new.ts"), b"untracked without HEAD\n").unwrap();
    let captured =
        capture_sources(unborn.path(), &CaptureRequest::new(Selection::Unstaged)).unwrap();
    assert_eq!(captured.changes().len(), 1);
    assert!(captured.changes()[0].before.is_none());
    assert_eq!(
        captured.changes()[0].after.as_ref().unwrap().bytes(),
        b"untracked without HEAD\n"
    );
}

#[test]
fn working_ranges_absence_and_configuration_survive_later_edits() {
    use anti_slop::{
        config::{Policy, SCHEMA_URL},
        workspace::SourceRange,
    };
    for selection in [
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        let root = repository();
        fs::remove_file(root.path().join("a.ts")).unwrap();
        fs::write(root.path().join("unicode.ts"), "é😀\n").unwrap();
        fs::write(root.path().join("empty"), b"").unwrap();
        let document = serde_json::json!({"$schema":SCHEMA_URL,"rules":{"R1":"block"},"overrides":[{"include":["**/*.ts"],"rules":{"R1":"off"}}]}).to_string();
        fs::write(root.path().join(".anti-slop.jsonc"), &document).unwrap();
        let request = CaptureRequest {
            context: vec!["unicode.ts".into(), "./unicode.ts".into()],
            ..CaptureRequest::new(selection)
        };
        let captured = capture_sources(root.path(), &request).unwrap();
        assert_eq!(captured.context().len(), 1);
        let deletion = captured
            .changes()
            .iter()
            .find(|c| c.path == "a.ts")
            .unwrap();
        assert!(deletion.after.is_none());
        assert_eq!(deletion.before.as_ref().unwrap().bytes(), b"head\n");
        let source = captured
            .changes()
            .iter()
            .find(|c| c.path == "unicode.ts")
            .unwrap()
            .after
            .as_ref()
            .unwrap();
        assert_ne!(source.generation, captured.context()[0].generation);
        assert_eq!(source.bytes(), captured.context()[0].bytes());
        let mut range = SourceRange {
            path: "unicode.ts".into(),
            role: Role::Working,
            generation: source.generation.clone(),
            range: 0..6,
        };
        assert_eq!(captured.slice(&range).unwrap(), "é😀".as_bytes());
        range.range = 7..7;
        assert_eq!(captured.slice(&range).unwrap(), b"");
        range.range = 0..8;
        assert!(captured.slice(&range).is_err());
        range.range = 0..6;
        range.role = Role::Context;
        assert!(captured.slice(&range).is_err());
        range.role = Role::Working;
        let empty = captured
            .changes()
            .iter()
            .find(|c| c.path == "empty")
            .unwrap()
            .after
            .as_ref()
            .unwrap();
        assert_eq!(
            captured
                .slice(&SourceRange {
                    path: "empty".into(),
                    role: Role::Working,
                    generation: empty.generation.clone(),
                    range: 0..0
                })
                .unwrap(),
            b""
        );
        let ts = captured
            .configuration()
            .resolve(Path::new("unicode.ts"))
            .unwrap();
        let php = captured
            .configuration()
            .resolve(Path::new("sample.php"))
            .unwrap();
        assert_eq!(ts.view().source, php.view().source);
        assert_eq!(ts.view().rules[0].policy, Policy::Off);
        assert_eq!(php.view().rules[0].policy, Policy::Block);
        fs::write(root.path().join("unicode.ts"), b"later\n").unwrap();
        fs::write(
            root.path().join(".anti-slop.jsonc"),
            document.replace("block", "advisory"),
        )
        .unwrap();
        let next = capture_sources(root.path(), &request).unwrap();
        assert_ne!(
            captured.configuration().generation(),
            next.configuration().generation()
        );
        assert!(next.slice(&range).is_err());
        fs::remove_file(root.path().join("unicode.ts")).unwrap();
        fs::remove_file(root.path().join(".anti-slop.jsonc")).unwrap();
        assert_eq!(captured.slice(&range).unwrap(), "é😀".as_bytes());
        assert_eq!(
            captured.configuration().captured_bytes(),
            Some(document.as_bytes())
        );
        assert_eq!(
            captured
                .configuration()
                .resolve(Path::new("sample.php"))
                .unwrap()
                .view()
                .rules[0]
                .policy,
            Policy::Block
        );
    }
}

#[test]
fn empty_unstaged_capture_identity_still_binds_the_observed_index_generation() {
    let root = repository();
    let first = capture_sources(root.path(), &CaptureRequest::new(Selection::Unstaged)).unwrap();
    assert!(first.changes().is_empty());
    fs::write(
        root.path().join("a.ts"),
        b"different index and equal working\n",
    )
    .unwrap();
    git(root.path(), &["add", "a.ts"]);
    let next = capture_sources(root.path(), &CaptureRequest::new(Selection::Unstaged)).unwrap();
    assert!(next.changes().is_empty());
    assert_ne!(
        first.generation(),
        next.generation(),
        "Empty selection is not absence of an observed source generation"
    );
}

#[test]
fn working_and_untracked_mutations_retry_without_combining_views() {
    controlled_git(
        "working-mutation",
        r#"
case " $* " in
  *" diff-tree "*)
    /usr/bin/git "$@" || exit 90
    if [ -f enumerated ] && [ ! -f intervened ]; then
      printf 'mutated working\n' > a.ts
      touch intervened
    fi
    touch enumerated
    exit 0
    ;;
esac
"#,
    );
    controlled_git(
        "untracked-added",
        r#"
case " $* " in
  *" ls-files --others "*)
    /usr/bin/git "$@" || exit 90
    if [ ! -f intervened ]; then
      printf 'created during acquisition\n' > created.ts
      touch intervened
    fi
    exit 0
    ;;
esac
"#,
    );
    controlled_git(
        "working-unstable",
        r#"
case " $* " in
  *" diff-index "*)
    /usr/bin/git "$@" || exit 90
    printf 'changing\n' >> a.ts
    exit 0
    ;;
esac
"#,
    );
}

#[test]
fn explicit_base_is_resolved_once_even_when_the_reference_moves() {
    controlled_git(
        "base-pin",
        r#"
case " $* " in
  *" cat-file "*)
    if [ ! -f intervened ]; then
      touch intervened
      /usr/bin/git -c user.name=Fixture -c user.email=fixture@example.invalid commit-tree 'HEAD^{tree}' -p HEAD -m advanced > advanced-head || exit 90
      /usr/bin/git update-ref refs/heads/moving "$(cat advanced-head)" || exit 91
    fi
    ;;
esac
"#,
    );
}

#[test]
fn expanding_untracked_inventory_has_the_same_bounded_process_cleanup() {
    controlled_git(
        "untracked-inventory",
        r#"
case " $* " in
  *" ls-files --others "*)
    echo $$ > owned-pid
    exec /usr/bin/yes expanding-untracked
    ;;
esac
"#,
    );
}

#[test]
fn working_inputs_apply_limits_and_exclusions_before_retaining_untracked_content() {
    use anti_slop::workspace::{CaptureError, MAX_FILE_BYTES};
    let root = repository();
    fs::write(root.path().join("huge"), vec![b'x'; MAX_FILE_BYTES + 1]).unwrap();
    for selection in [
        Selection::Unstaged,
        Selection::Local,
        Selection::Base("HEAD".into()),
    ] {
        assert!(matches!(
            capture_sources(root.path(), &CaptureRequest::new(selection.clone())),
            Err(CaptureError::Limit(_))
        ));
        let result = capture_sources(
            root.path(),
            &CaptureRequest {
                exclusions: vec!["./huge".into()],
                ..CaptureRequest::new(selection)
            },
        );
        let Err(CaptureError::Excluded(observed)) = result else {
            panic!("{result:?}")
        };
        assert_eq!(
            observed,
            vec![anti_slop::workspace::ObservedInput {
                path: "huge".into(),
                role: Role::Working
            }]
        );
    }
    git(root.path(), &["config", "core.filemode", "false"]);
    fs::remove_file(root.path().join("huge")).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(root.path().join("a.ts"), fs::Permissions::from_mode(0o755)).unwrap();
    let captured = capture_sources(root.path(), &CaptureRequest::new(Selection::Local)).unwrap();
    assert_eq!(captured.changes().len(), 1);
    assert_eq!(
        captured.changes()[0].before.as_ref().unwrap().git_mode,
        Some(anti_slop::workspace::GitMode::Regular)
    );
    assert_eq!(
        captured.changes()[0].after.as_ref().unwrap().git_mode,
        Some(anti_slop::workspace::GitMode::Executable)
    );
    assert_eq!(
        captured.changes()[0].before.as_ref().unwrap().bytes(),
        captured.changes()[0].after.as_ref().unwrap().bytes()
    );
}
