# AS-M004 P04 — whole-process memory observer correction

Complete implementation report for packet AS-M004-P04 revision 1 under mission AS-M004 revision 2. Outcome: the single finding from the independent final-r1 review is corrected, proved with real behavioral RED/GREEN, and the affected resource evidence is renewed. No production code, dependency, prior evidence, verdict or mission record was modified.

## Assignment and exact state

Actor `as-m004-durable:s1794916284.2288889.8486`, role implementer. Repository `anti-slop` at `/home/abdwhb/engineering/firstmate-orchestration-test/data/as-m004-durable/worktree`, branch `fm/as-m004-durable`, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`. All AS-M004 implementation remains uncommitted exactly as the final-r1 review recorded it; `base..HEAD` is still empty and no commit, staging, push, PR, merge or cleanup occurred. `git diff --cached` is empty and `git diff --check` reports no whitespace error.
`git status --short` before and after this packet is the same 12 entries: modified `Cargo.lock`, `Cargo.toml`, `README.md`, `docs/IMPLEMENTATION_ROADMAP.md`, `src/lib.rs`; untracked `.missions/AS-M004/`, `docs/missions/AS-M004/`, `src/records/mod.rs`, `tests/records.rs`, `tests/records_demo.rs`, `tests/records_measure.rs`, `tests/support/records/`. The only deliverable content changed in this packet is `tests/records_measure.rs`; everything else under `.missions/AS-M004/` that changed is new `evidence/P04/` material.
Preservation was verified by hash, not by assertion. These files still match the final-r1 `reviewedPaths` values byte for byte: `src/records/mod.rs` `5a075586…726992d`, `src/lib.rs` `21936cdd…77a835f`, `tests/records.rs` `a14a8756…ff447db2`, `tests/records_demo.rs` `b45bc980…bebd1a49`, `tests/support/records/mod.rs` `3153e82b…472973b4`, `tests/support/records/io_fault.c` `23ea7d4d…7b038b8c7`, `Cargo.toml` `1ce7a7ff…73a8bcb`, `Cargo.lock` `ee591e40…4c15381`, `README.md` `31ee116e…068c1d`, `docs/IMPLEMENTATION_ROADMAP.md` `4d9e6480…8f6f6f3`, `docs/missions/AS-M004/r2.md` `3940a139…e25e294`.
The project MDD bundle validator was run on `.missions/AS-M004/state.json` before any work: `node …/mdd-bundle/scripts/validate.mjs .missions/AS-M004/state.json --json` returned `{"valid":true,"issues":[]}`, exit 0. It was run again after the packet evidence was written, with the same result. The state and history files were not edited; registering this evidence and the packet result remains Firstmate's authority.

## Established diagnosis, confirmed live

The final-r1 finding was that `tree_rss` walked only `/proc/<pid>/task/<pid>/children`, which is the leader thread's child list, while the libtest worker thread launches the capture's Git children. That structure is visible in the preserved pre-packet source `measurement-source-before.raw.rs.txt` (15,382 bytes, SHA-256 `73ffa7ac07559ea87f2d91f70850fa6a9957e3235a5c5b80aa7976858dfa5379`).
This packet reproduced the parentage independently. During the unchanged real capture fixture, the corrected observer reported Git PID 2382930 listed under TID 2372415 of process 2372414 on the one-file journey, PID 2383024 under the same TID on the five-file journey, and PID 2383089 under the same TID on the large-batch journey, always with `git_via_leader_peak=0`. The final-r1 review observed the same relation (Git 2116266 under TID 2106694 of process 2106689). The omission is therefore systematic for capture journeys, not a missed 1 ms peak.
A useful kernel detail was also established while building the regression: when a thread that owns a child exits, the kernel reparents its children to another thread in the same thread group, so the leader's list can stay empty while the process plainly owns children. A process-wide union over every thread's children list follows that movement, and the regression currently keeps its launcher thread alive so the nonleader parentage remains observable for the whole assertion.

## RED — actual failing behavior through the consuming module

The regression lives in `tests/records_measure.rs`, the real resource protocol target, and calls the actual observer rather than a copy. It launches four real `git cat-file --batch` children from a spawned nonleader thread, keeps them blocked on their piped stdin, records the TID from `/proc/thread-self` and each kernel-assigned `Child::id`, waits until the kernel lists all four under that TID, and then asserts whole-tree coverage. The launcher stays alive behind a release channel, and a drop guard kills and reaps every child on success and on panic.
Command: `/home/abdwhb/.cargo/bin/cargo test --frozen --test records_measure -- --exact tree_rss_includes_git_children_launched_from_a_nonleader_thread --nocapture`. Exit 101. Raw output is `red-01-thread-child-omitted.txt`; the decisive lines are:

```
thread_child process=2344558 launcher_tid=2344587 git_pids=[2344588, 2344589, 2344590, 2344591] git_rss=18546688 own_rss=7073792 observed_tree_rss=7073792 leader_children=""
tree_rss 7073792 must include the 18546688-byte Git children [2344588, 2344589, 2344590, 2344591] launched by nonleader TID 2344587; own process RSS is 7073792
```

Four children are held because a small Git process holds little resident memory; the omitted 18.5 MB then dwarfs ordinary process-RSS drift, so the assertion is decisive rather than marginal. It fails on the observer's actual omission, with the leader child list empty and the live children independently known, not on an import, type or fixture error. An earlier fixture attempt failed because the launcher exited before observation and the kernel reparented the children; that was repaired before counting RED, as the packet requires, and is described in `regression-evidence.md`.

## Correction

`tree_rss` is now implemented through one authoritative observer, `owned_tree_pids`, which visits each owned process, reads every `/proc/<pid>/task/<tid>/children` entry, recurses, and deduplicates by process id while recording the TID that listed each child. `tree_rss` sums `VmRSS` once per distinct process id and skips a process that vanished between listing and reading, treating that death race as a sampling limit rather than as unavailable evidence. No thread is counted as another address space, no observer copy exists in the test, and no dependency, supervisor or framework was added.
The measured source grew from 15,382 to 24,827 bytes: the observer correction, the thread-child regression and fixture guard, the `process_comm` coverage probe, a coverage assertion in the real capture journeys, and a mutex shared by the two tests. The mutex exists because the packet forbids keeping an artificial blocked Git fixture resident during normal workload measurements; with it the regression and the protocol can never overlap even when CI runs the test binary with default parallelism.
`tests/records_measure.rs` final SHA-256 is `1aa6fdeb1e7779055f0141fbfd252f1ef5062bc47b8073052a7df147b298a6fd`, preserved as `measurement-source-after.raw.rs.txt`; `measurement-source.diff.txt` is the exact unified diff against the preserved pre-packet source, so every change is reviewable without the conversation.

## GREEN and real Git coverage

Command: same focused command as RED. Exit 0, one test passed in 0.04 s; raw output `green-01-thread-child-observed.txt`.

```
thread_child process=2356777 launcher_tid=2356806 git_pids=[2356807, 2356808, 2356809, 2356810] git_rss=13115392 own_rss=7028736 observed_tree_rss=20770816 observed_members=[(2356777, None), (2356807, Some(2356806)), (2356808, Some(2356806)), (2356809, Some(2356806)), (2356810, Some(2356806))] leader_children=""
test tree_rss_includes_git_children_launched_from_a_nonleader_thread ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.04s
```

The observer reports each Git child under the nonleader TID that actually launched it, and the summed tree (20,770,816) exceeds own RSS plus the independently read child RSS (20,144,128). Because the RED revision could not reference a membership relation that did not yet exist, RED asserted only numeric coverage; the published revision adds the membership assertion shown here, which strengthens the same claim rather than replacing it. `rustfmt` reflowed the file before the published hash was taken, so the RED-stage source text is not byte-recoverable; both the pre-packet source and the final source are preserved, and the RED log is exactly the output of the pre-fix observer.
Coverage was then verified on the unchanged real capture fixture, not only on the controlled regression: all three combined journeys report `git_children_peak=1`, `git_via_leader_peak=0`, `git_via_other_thread_peak=1`, and a concrete thread-launched Git child. The protocol asserts `git_children_peak > 0` for each combined journey, so an unobserved child cannot be reported as a complete tree measurement. `tree_rss_peak` rises accordingly from the save-only 22.16–24.46 MB band to 34,279,424 bytes for one and five files and 52,887,552 bytes for the large batch.

## Renewed observations

Final measurement command: `/home/abdwhb/.cargo/bin/cargo test --frozen --test records_measure -- --nocapture --test-threads=1`. Exit 0, 2 tests passed in 10.41 s. Raw output `measure-current.txt`; current protocol and limits in `resource-protocol-current.md`. Host `Linux 7.0.0-42-generic x86_64 GNU/Linux`, 4 available CPUs, load average `0.63 0.92 1.13`, warm caches, Rust 1.96.1, same fixtures and run counts as P03.
Latency stays in the P03 band: one-file capture-only p50/p95 61,675/75,362 µs over 30 runs, five-file 89,659/100,420 µs, one-file save+commit 393/466 µs and five-file 699/1,470 µs over 20 runs, one-file capture+save+read 62,469/90,485 µs and five-file 92,720/148,241 µs over 20 runs, reopen 288/360 µs, large-batch capture+save 614,575 µs and save+commit 145,843 µs.
Storage is unchanged in kind: retained 90,456 for one file, 102,816 for five files, 181,096 for the 480-file metadata case, 1,149,296 for the 1 MiB file, 16,970,168 for the 8 MiB batch and 3,316,416 for the held-reader two-save case; the sampled database+WAL+SHM+journal peak equals the final retained size in every case, which is the expected 1 ms lower bound for these fast writes. The process ended at `VmHWM` 31,848 KiB and `VmRSS` 20,748 KiB.
Whole-owned-process memory is now complete for the capture journeys: 34,279,424 bytes for one and five files and 52,887,552 bytes for the large batch, including the thread-launched Git children. Save-only peaks (22,163,456 to 24,461,312 bytes) are unchanged in meaning because those phases spawn no child. This explicitly supersedes the P03 statement that the tree sampler included live Git children and the P03 combined large-batch value of 32,600,064 bytes; the omitted ~20.3 MB was observer omission, not product-memory growth. P03 logs, protocol and reports remain untouched as historical observations.

## Applicability of prior evidence

The behavioral, integrity, schema, complete-or-absent, post-commit, admission, concurrency, effective-path, dependency and demonstration evidence from P01–P03C1 is unaffected: this packet touches no production source, dependency, manifest or other test, and that was verified by the hashes above. The 38 records tests and two demonstration tests still pass unchanged in the full run. P03's latency and storage figures remain directionally valid and are renewed here in one current snapshot; no claim in them depended on the observer. Only the whole-process memory claims change, and only for journeys that launch Git: those are replaced by the values in `resource-protocol-current.md`, whose heading, title and supersession section make the replacement explicit.
The current nonvisual demonstration story is also unaffected. It shows select `a.ts`, retain `guide.ts` as context, save, refuse a second capture under a zero budget while the first stays readable, delete the origins, and read exact bytes/roles/generations in a fresh process. Its behavior is exercised by `tests/records_demo.rs`, which is byte-identical to the reviewed version and passes in the full run; `evidence/P03C1/demo-output.txt` remains the recorded output.

## Commands and results

- `node …/mdd-bundle/scripts/validate.mjs .missions/AS-M004/state.json --json` — before and after the packet, exit 0, `valid:true`, `issues:[]`, state unchanged.
- Focused RED `cargo test --frozen --test records_measure -- --exact tree_rss_includes_git_children_launched_from_a_nonleader_thread --nocapture` — exit 101, intended assertion failure, log preserved.
- Focused GREEN, same command — exit 0, membership and numeric coverage pass, log preserved.
- `cargo fmt --all` then `cargo fmt --all -- --check` — formatted and clean; `git status` confirms no other file changed.
- `cargo clippy --frozen --test records_measure -- -D warnings` — exit 0.
- Final measurement run above — exit 0, 2 passed, full raw output preserved.
- `PATH=/home/abdwhb/.cargo/bin:$PATH CI_BASE_SHA=45dac3c241338c94166369fe821fe1d3499cf974 bash scripts/ci/check.sh` — exit 0 once after stabilization. Because the script runs under `set -euo pipefail`, exit 0 covers formatting, 19 CI policy unit tests, whitespace policy, frozen workspace build, all-target tests (CLI 11, config 19, output 3, records 38, demonstration 2, resource protocol 2, schema 3, workspace 34 passed with 2 pre-existing ignored), doctest target, Clippy with `-D warnings`, generated-schema check and Node editor-service conformance. Raw output `ci-check.txt`.
- `git status --short`, `git diff --cached`, `git diff --check`, `git rev-parse HEAD` and SHA-256 comparison of the reviewed paths — scope, preservation and cleanliness confirmed as reported above.

## Checks not run

No full-suite repetition between edits; only the focused test ran during RED/GREEN and the single required CI-equivalent script ran after stabilization. No networked registry, dependency installation or version selection was performed, and no dependency or lockfile was touched. GitHub Actions against a committed or pushed head, cold-disk measurement, device- or power-loss durability, lying-fsync behavior, other SQLite builds or filesystems, and ADR-007 future budgets were not exercised and are not claimed.

## Remaining limits and uncertainty

The observer still samples at 1 ms, so a child that is born and exits between samples is missed; absence in a sample is a sampling limitation, not proof of absence. Per-process RSS sums count shared pages more than once, biasing the tree total upward. A process that dies between listing and reading contributes nothing, and PID reuse inside one window is not observed but not excluded. Measurements are one warm-cache host, one load snapshot, a debug-profile build and small run counts, and they establish no future product budget. The RED-stage test text is not byte-preserved (only the pre-fix source and final source are), so a byte-exact rereun of RED would require reconstructing that intermediate revision; the captured RED log is genuine and predates the fix. No product defect or budget overrun is inferred from the corrected values: the increase is exactly the previously omitted Git children.

## Handoff

Status: `done: candidate ready for examination`. This packet's deliverable is the corrected measurement observer plus renewed, source-linked resource evidence, and it is complete and self-contained in `evidence/P04/`. This is a candidate only: it is not packet acceptance, mission acceptance, publication, delivery or closure. Firstmate examines it; a separate fresh Codex context remains responsible for final whole-mission acceptance. No commit, staging, push, PR, merge, No-Mistakes run, deployment, state edit or cleanup was performed.
