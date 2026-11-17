//! Reproducible real-data observation of capture, save, commit, read and storage cost.
//!
//! Run alone with:
//! `cargo test --frozen --test records_measure -- --nocapture --test-threads=1`

#[path = "support/records/mod.rs"]
mod support;

use anti_slop::{
    config::MAX_FILE_BYTES as MAX_CONFIG_BYTES,
    records::{DEFAULT_BUDGET_BYTES, Store},
    workspace::{
        Capture, CaptureRequest, MAX_FILE_BYTES, MAX_FILES, MAX_TOTAL_BYTES, Selection, capture,
    },
};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

/// The protocol and the thread-child regression keep one process tree; running them
/// concurrently would mix the regression's blocked Git child into the protocol's peaks.
static RESOURCE_PROTOCOL_LOCK: Mutex<()> = Mutex::new(());

fn resource_protocol_guard() -> MutexGuard<'static, ()> {
    RESOURCE_PROTOCOL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

fn rss_bytes(pid: u32) -> Option<u64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmRSS:")
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|kib| kib * 1024)
    })
}

/// Every process currently reported inside the owned tree, paired with the thread that
/// listed it (`None` for the root). A child belongs to the task that created it, so every
/// `/proc/<pid>/task/<tid>/children` list is read, not only the leader thread's.
fn owned_tree_pids(pid: u32) -> Vec<(u32, Option<u32>)> {
    let mut pids = vec![(pid, None)];
    let mut index = 0;
    while index < pids.len() {
        let current = pids[index].0;
        if let Ok(tasks) = fs::read_dir(format!("/proc/{current}/task")) {
            for task in tasks.flatten() {
                let Some(tid) = task
                    .file_name()
                    .to_str()
                    .and_then(|name| name.parse::<u32>().ok())
                else {
                    continue;
                };
                let Ok(children) =
                    fs::read_to_string(format!("/proc/{current}/task/{tid}/children"))
                else {
                    continue;
                };
                for child in children.split_whitespace() {
                    if let Ok(child) = child.parse::<u32>()
                        && !pids.iter().any(|(known, _)| *known == child)
                    {
                        pids.push((child, Some(tid)));
                    }
                }
            }
        }
        index += 1;
    }
    pids
}

/// Sum of each owned process's RSS, read once per process id. A process that exited between
/// being listed and being read contributes nothing; that death race is not evidence that
/// observation was unavailable.
fn tree_rss(pid: u32) -> u64 {
    owned_tree_pids(pid)
        .iter()
        .filter_map(|(pid, _)| rss_bytes(*pid))
        .sum()
}

fn process_comm(pid: u32) -> Option<String> {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .ok()
        .map(|comm| comm.trim().to_owned())
}

/// The kernel names the current task under `/proc/thread-self`, so this is the TID that
/// owns any child this thread launches, not merely the thread-group leader id.
fn current_thread_id() -> u32 {
    fs::read_link("/proc/thread-self")
        .ok()
        .and_then(|link| link.file_name()?.to_str()?.parse().ok())
        .expect("Linux exposes the current TID through /proc/thread-self")
}

fn child_listed_under(process: u32, tid: u32, child: u32) -> bool {
    fs::read_to_string(format!("/proc/{process}/task/{tid}/children"))
        .map(|children| {
            children
                .split_whitespace()
                .any(|candidate| candidate.parse::<u32>() == Ok(child))
        })
        .unwrap_or(false)
}

/// A real Git child launched from a nonleader thread that stays alive until this guard is
/// dropped. The kernel reparents a thread's children when that thread exits, so the
/// launcher must remain alive for the parentage to stay observable; dropping the guard
/// signals the launcher, which kills and reaps Git even when an assertion fails.
struct ThreadHeldGit {
    launcher_tid: u32,
    git: Vec<u32>,
    release: Option<mpsc::Sender<()>>,
    launcher: Option<thread::JoinHandle<()>>,
}

impl ThreadHeldGit {
    /// Small Git processes hold little resident memory, so several are held at once: the
    /// omitted memory must exceed ordinary process-RSS drift for the regression to be
    /// decisive on both the old and the corrected observer.
    const CHILDREN: usize = 4;
}

impl Drop for ThreadHeldGit {
    fn drop(&mut self) {
        drop(self.release.take());
        if let Some(launcher) = self.launcher.take() {
            let _ = launcher.join();
        }
    }
}

fn own_status(field: &str) -> u64 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix(field)
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|value| value.parse::<u64>().ok())
            })
        })
        .unwrap_or(0)
}

fn percentile(mut values: Vec<Duration>, percentile: f64) -> Duration {
    values.sort();
    let index = ((values.len() as f64 * percentile).ceil() as usize).saturating_sub(1);
    values[index.min(values.len() - 1)]
}

fn micros(duration: Duration) -> u128 {
    duration.as_micros()
}

fn report(name: &str, values: &[Duration]) {
    let p50 = percentile(values.to_vec(), 0.50);
    let p95 = percentile(values.to_vec(), 0.95);
    println!(
        "journey={name} runs={} p50_us={} p95_us={} raw_us={:?}",
        values.len(),
        micros(p50),
        micros(p95),
        values
            .iter()
            .map(|value| micros(*value))
            .collect::<Vec<_>>()
    );
}

fn one_file_repository() -> TempDir {
    let repository = support::repository();
    support::write(repository.path(), "a.ts", b"one line changed\n");
    repository
}

fn five_file_repository() -> (TempDir, CaptureRequest) {
    let repository = support::repository();
    for file in 0..5 {
        let name = format!("five-{file}.txt");
        let initial = (0..40)
            .map(|line| format!("initial {file} {line}\n"))
            .collect::<String>();
        support::write(repository.path(), &name, initial.as_bytes());
    }
    support::git(repository.path(), &["add", "."]);
    support::git(repository.path(), &["commit", "-qm", "five files"]);
    for file in 0..5 {
        let name = format!("five-{file}.txt");
        let changed = (0..40)
            .map(|line| format!("changed {file} {line}\n"))
            .collect::<String>();
        support::write(repository.path(), &name, changed.as_bytes());
    }
    (repository, CaptureRequest::new(Selection::Local))
}

fn metadata_repository() -> TempDir {
    let repository = support::repository();
    for index in 0..480 {
        support::write(repository.path(), &format!("meta-{index}.txt"), b"x\n");
    }
    repository
}

fn large_single_repository() -> TempDir {
    let repository = support::repository();
    support::write(
        repository.path(),
        "large-single.bin",
        &vec![b'z'; MAX_FILE_BYTES],
    );
    repository
}

fn large_batch_repository() -> TempDir {
    let repository = support::repository();
    for index in 0..8 {
        support::write(
            repository.path(),
            &format!("large-{index}.bin"),
            &vec![b'y'; MAX_FILE_BYTES],
        );
    }
    repository
}

#[test]
fn tree_rss_includes_git_children_launched_from_a_nonleader_thread() {
    let _protocol = resource_protocol_guard();
    let repository = support::repository();
    let process = std::process::id();
    let (ready_sender, ready_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel::<()>();
    let root = repository.path().to_owned();
    let launcher = thread::spawn(move || {
        let tid = current_thread_id();
        let mut children = Vec::with_capacity(ThreadHeldGit::CHILDREN);
        for _ in 0..ThreadHeldGit::CHILDREN {
            children.push(
                Command::new("git")
                    .arg("-C")
                    .arg(&root)
                    .args(["cat-file", "--batch"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("a real Git process is required for the child observation"),
            );
        }
        ready_sender
            .send((tid, children.iter().map(std::process::Child::id).collect()))
            .expect("the test receives the launched Git children");
        let _ = release_receiver.recv();
        for child in &mut children {
            let _ = child.kill();
            let _ = child.wait();
        }
    });
    let (launcher_tid, git) = ready_receiver
        .recv()
        .expect("the launcher thread reports its live Git children");
    let held = ThreadHeldGit {
        launcher_tid,
        git,
        release: Some(release_sender),
        launcher: Some(launcher),
    };
    assert_ne!(
        held.launcher_tid, process,
        "the regression must launch Git from a nonleader thread"
    );
    assert!(
        held.git.iter().all(|pid| *pid != process),
        "every Git child runs in its own process image"
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut listed = false;
    while Instant::now() < deadline {
        if held
            .git
            .iter()
            .all(|git| child_listed_under(process, held.launcher_tid, *git))
        {
            listed = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        listed,
        "the live Git children {:?} must be listed under TID {} of process {process}",
        held.git, held.launcher_tid
    );

    let leader_children =
        fs::read_to_string(format!("/proc/{process}/task/{process}/children")).unwrap_or_default();
    let membership = owned_tree_pids(process);
    let own_before = rss_bytes(process).expect("the test process RSS is observable");
    let child_before: u64 = held.git.iter().filter_map(|pid| rss_bytes(*pid)).sum();
    let observed = tree_rss(process);
    let own_after = rss_bytes(process).expect("the test process RSS stays observable");
    let child_after: u64 = held.git.iter().filter_map(|pid| rss_bytes(*pid)).sum();
    let own = own_before.min(own_after);
    let child_rss = child_before.min(child_after);
    println!(
        "thread_child process={process} launcher_tid={} git_pids={:?} git_rss={child_rss} own_rss={own} observed_tree_rss={observed} observed_members={membership:?} leader_children={:?}",
        held.launcher_tid,
        held.git,
        leader_children.trim()
    );
    assert!(
        child_rss > 0,
        "the live Git children must hold resident memory"
    );
    assert!(
        held.git.iter().all(|git| membership
            .iter()
            .any(|(pid, listed_under)| pid == git && *listed_under == Some(held.launcher_tid))),
        "the observer must report every Git child under the nonleader TID that launched it: {membership:?}"
    );
    assert!(
        observed >= own.saturating_add(child_rss),
        "tree_rss {observed} must include the {child_rss}-byte Git children {:?} launched by nonleader TID {}; own process RSS is {own}",
        held.git,
        held.launcher_tid
    );
}

#[test]
fn capture_save_commit_read_resource_protocol() {
    let _protocol = resource_protocol_guard();
    println!(
        "host={} cpus={} loadavg={:?}",
        std::env::consts::OS,
        thread::available_parallelism().map_or(0, |value| value.get()),
        fs::read_to_string("/proc/loadavg").ok().map(|value| value
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" "))
    );
    println!(
        "bounds files={MAX_FILES} file_bytes={MAX_FILE_BYTES} total_bytes={MAX_TOTAL_BYTES} config_bytes={MAX_CONFIG_BYTES} budget={DEFAULT_BUDGET_BYTES}"
    );

    let one = one_file_repository();
    let (five, five_request) = five_file_repository();
    let metadata = metadata_repository();
    let large = large_single_repository();
    let large_batch = large_batch_repository();
    let one_request = CaptureRequest::new(Selection::Local);
    let metadata_request = CaptureRequest::new(Selection::Local);
    let large_request = CaptureRequest::new(Selection::Local);

    let one_capture: Capture = capture(one.path(), &one_request).unwrap();
    let five_capture: Capture = capture(five.path(), &five_request).unwrap();
    let metadata_capture: Capture = capture(metadata.path(), &metadata_request).unwrap();
    let large_capture: Capture = capture(large.path(), &large_request).unwrap();
    let large_batch_capture: Capture = capture(large_batch.path(), &large_request).unwrap();
    println!(
        "captures one_bytes={} five_changes={} five_bytes={} metadata_changes={} metadata_bytes={} large_bytes={}",
        one_capture.stats().retained_bytes,
        five_capture.changes().len(),
        five_capture.stats().retained_bytes,
        metadata_capture.changes().len(),
        metadata_capture.stats().retained_bytes,
        large_capture.stats().retained_bytes
    );
    println!(
        "captures large_batch_changes={} large_batch_bytes={}",
        large_batch_capture.changes().len(),
        large_batch_capture.stats().retained_bytes
    );

    let mut one_capture_times = Vec::new();
    for _ in 0..30 {
        let started = Instant::now();
        let captured = capture(one.path(), &one_request).unwrap();
        one_capture_times.push(started.elapsed());
        std::hint::black_box(captured);
    }
    report("one_file_capture_only", &one_capture_times);

    let mut five_capture_times = Vec::new();
    for _ in 0..30 {
        let started = Instant::now();
        let captured = capture(five.path(), &five_request).unwrap();
        five_capture_times.push(started.elapsed());
        std::hint::black_box(captured);
    }
    report("five_file_capture_only", &five_capture_times);

    for (name, repository, request, captured) in [
        ("one_file", one.path(), &one_request, &one_capture),
        ("five_file", five.path(), &five_request, &five_capture),
    ] {
        let mut open_times = Vec::new();
        let mut save_times = Vec::new();
        let mut read_times = Vec::new();
        let mut total_times = Vec::new();
        for _ in 0..20 {
            let directory = TempDir::new().unwrap();
            let database = directory.path().join("records.sqlite3");
            let started = Instant::now();
            let mut store = Store::open(&database).unwrap();
            open_times.push(started.elapsed());

            let total_started = Instant::now();
            let journey_capture = capture(repository, request).unwrap();
            let save_started = Instant::now();
            let saved = store.save_capture(&journey_capture).unwrap();
            save_times.push(save_started.elapsed());
            let read_started = Instant::now();
            store.read_capture(saved.id).unwrap();
            read_times.push(read_started.elapsed());
            total_times.push(total_started.elapsed());
            std::hint::black_box(captured);
        }
        report(&format!("{name}_open"), &open_times);
        report(&format!("{name}_save_commit"), &save_times);
        report(&format!("{name}_read"), &read_times);
        report(&format!("{name}_capture_save_read"), &total_times);
    }

    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    store.save_capture(&one_capture).unwrap();
    drop(store);
    let mut reopen_times = Vec::new();
    for _ in 0..20 {
        let started = Instant::now();
        let store = Store::open(&database).unwrap();
        reopen_times.push(started.elapsed());
        drop(store);
    }
    report("reopen", &reopen_times);

    observe_peak("one_file", &one_capture);
    observe_peak("five_file", &five_capture);
    observe_peak("metadata_heavy", &metadata_capture);
    observe_peak("large_single", &large_capture);
    observe_peak("large_batch", &large_batch_capture);
    observe_combined("one_file", one.path(), &one_request);
    observe_combined("five_file", five.path(), &five_request);
    observe_combined("large_batch", large_batch.path(), &large_request);
    observe_held_reader(large.path(), &large_request);
    println!(
        "memory self_vmhwm_kib={} self_vmrss_kib={}",
        own_status("VmHWM:"),
        own_status("VmRSS:")
    );
}

fn observe_peak(name: &str, captured: &Capture) {
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let sampling = Arc::new(AtomicBool::new(true));
    let peak_storage = Arc::new(AtomicU64::new(0));
    let peak_tree = Arc::new(AtomicU64::new(0));
    let flag = Arc::clone(&sampling);
    let storage = Arc::clone(&peak_storage);
    let tree = Arc::clone(&peak_tree);
    let monitor_database = database.clone();
    let pid = std::process::id();
    peak_storage.store(storage_files(&database), Ordering::Relaxed);
    peak_tree.store(tree_rss(pid), Ordering::Relaxed);
    let monitor = thread::spawn(move || {
        while flag.load(Ordering::Relaxed) {
            storage.fetch_max(storage_files(&monitor_database), Ordering::Relaxed);
            tree.fetch_max(tree_rss(pid), Ordering::Relaxed);
            thread::sleep(Duration::from_millis(1));
        }
    });
    let started = Instant::now();
    let saved = store.save_capture(captured).unwrap();
    let save_time = started.elapsed();
    let usage = store.usage().unwrap();
    peak_storage.fetch_max(storage_files(&database), Ordering::Relaxed);
    peak_tree.fetch_max(tree_rss(pid), Ordering::Relaxed);
    sampling.store(false, Ordering::Relaxed);
    monitor.join().unwrap();
    println!(
        "peak case={name} payload={} admitted_limit={} retained={} save_us={} storage_peak={} storage_peak_over_retained={} tree_rss_peak={} aliases={} changes={}",
        captured.stats().retained_bytes,
        usage.limit,
        usage.retained,
        micros(save_time),
        peak_storage.load(Ordering::Relaxed),
        peak_storage
            .load(Ordering::Relaxed)
            .saturating_sub(usage.retained),
        peak_tree.load(Ordering::Relaxed),
        captured.context().len(),
        captured.changes().len()
    );
    std::hint::black_box(saved);
}

fn observe_combined(name: &str, repository: &Path, request: &CaptureRequest) {
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let sampling = Arc::new(AtomicBool::new(true));
    let peak_storage = Arc::new(AtomicU64::new(storage_files(&database)));
    let peak_tree = Arc::new(AtomicU64::new(tree_rss(std::process::id())));
    let peak_git = Arc::new(AtomicU64::new(0));
    let peak_git_leader = Arc::new(AtomicU64::new(0));
    let peak_git_thread = Arc::new(AtomicU64::new(0));
    let git_example = Arc::new(Mutex::new(None::<(u32, u32)>));
    let flag = Arc::clone(&sampling);
    let storage = Arc::clone(&peak_storage);
    let tree = Arc::clone(&peak_tree);
    let git = Arc::clone(&peak_git);
    let git_leader = Arc::clone(&peak_git_leader);
    let git_thread = Arc::clone(&peak_git_thread);
    let example = Arc::clone(&git_example);
    let monitor_database = database.clone();
    let pid = std::process::id();
    let monitor = thread::spawn(move || {
        while flag.load(Ordering::Relaxed) {
            storage.fetch_max(storage_files(&monitor_database), Ordering::Relaxed);
            tree.fetch_max(tree_rss(pid), Ordering::Relaxed);
            let (mut total, mut leader, mut other) = (0_u64, 0_u64, 0_u64);
            let mut sample = None;
            for (child, listed_under) in owned_tree_pids(pid) {
                if process_comm(child).as_deref() != Some("git") {
                    continue;
                }
                total += 1;
                match listed_under {
                    Some(tid) if tid == pid => leader += 1,
                    Some(tid) => {
                        other += 1;
                        sample = Some((child, tid));
                    }
                    None => {}
                }
            }
            git.fetch_max(total, Ordering::Relaxed);
            git_leader.fetch_max(leader, Ordering::Relaxed);
            git_thread.fetch_max(other, Ordering::Relaxed);
            if sample.is_some() {
                *example
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = sample;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });
    let started = Instant::now();
    let captured = capture(repository, request).unwrap();
    let saved = store.save_capture(&captured).unwrap();
    let elapsed = started.elapsed();
    peak_storage.fetch_max(storage_files(&database), Ordering::Relaxed);
    peak_tree.fetch_max(tree_rss(pid), Ordering::Relaxed);
    sampling.store(false, Ordering::Relaxed);
    monitor.join().unwrap();
    let example = *git_example
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    println!(
        "combined case={name} capture_save_us={} payload={} retained={} storage_peak={} tree_rss_peak={} git_children_peak={} git_via_leader_peak={} git_via_other_thread_peak={} git_child_listed_under={example:?} process_pid={pid}",
        micros(elapsed),
        captured.stats().retained_bytes,
        saved.usage.retained,
        peak_storage.load(Ordering::Relaxed),
        peak_tree.load(Ordering::Relaxed),
        peak_git.load(Ordering::Relaxed),
        peak_git_leader.load(Ordering::Relaxed),
        peak_git_thread.load(Ordering::Relaxed)
    );
    assert!(
        peak_git.load(Ordering::Relaxed) > 0,
        "the real capture journey must observe its Git children; an unobserved child is not a complete tree measurement"
    );
}

fn observe_held_reader(repository: &Path, request: &CaptureRequest) {
    let directory = TempDir::new().unwrap();
    let database = directory.path().join("records.sqlite3");
    let mut store = Store::open(&database).unwrap();
    let first = capture(repository, request).unwrap();
    store.save_capture(&first).unwrap();
    let reader = rusqlite::Connection::open(&database).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    let _: i64 = reader
        .query_row("SELECT count(*) FROM captures", [], |row| row.get(0))
        .unwrap();
    let sampling = Arc::new(AtomicBool::new(true));
    let peak_storage = Arc::new(AtomicU64::new(storage_files(&database)));
    let flag = Arc::clone(&sampling);
    let peak = Arc::clone(&peak_storage);
    let monitor_database = database.clone();
    let monitor = thread::spawn(move || {
        while flag.load(Ordering::Relaxed) {
            peak.fetch_max(storage_files(&monitor_database), Ordering::Relaxed);
            thread::sleep(Duration::from_millis(1));
        }
    });
    let started = Instant::now();
    let _saved = store.save_capture(&first).unwrap();
    let saved_second = store.save_capture(&first).unwrap();
    let elapsed = started.elapsed();
    peak_storage.fetch_max(storage_files(&database), Ordering::Relaxed);
    sampling.store(false, Ordering::Relaxed);
    monitor.join().unwrap();
    reader.execute_batch("ROLLBACK").unwrap();
    println!(
        "peak case=held_reader payload={} retained={} save_us={} storage_peak={}",
        first.stats().retained_bytes,
        saved_second.usage.retained,
        micros(elapsed),
        peak_storage.load(Ordering::Relaxed)
    );
    std::hint::black_box((repository, request));
}
