# AS-M004 P03 implementation report

done: candidate ready for examination

This is the final packet candidate for mission AS-M004. It does not accept the mission, change state, commit, publish, merge or authorize No-Mistakes. Firstmate examines this result; Codex remains the independent final whole-mission reviewer.

## Identity and Git provenance

- Role: implementer for packet `AS-M004-P03` revision 1, mission `AS-M004` revision 2.
- Execution identity: `as-m004-durable:s1794906814.562141.27268`, continued OpenCode session, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no verified selectable effort variant. Provider quota and inference identity remain unverified.
- Base and HEAD unchanged: `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`). All mission work remains uncommitted, intentionally retained. No commit, stash, reset, clean, dependency/manifest/lock change, installation or other-home write was performed.
- Final status:

```text
 M Cargo.lock
 M Cargo.toml
 M README.md
 M docs/IMPLEMENTATION_ROADMAP.md
 M src/lib.rs
?? .missions/AS-M004/
?? docs/missions/AS-M004/
?? src/records/mod.rs
?? tests/records.rs
?? tests/records_demo.rs
?? tests/records_measure.rs
?? tests/support/records/
```

- `Cargo.toml`/`Cargo.lock` preserve the single qualified `rusqlite 0.41.1` graph from P01/P01C1; P03 changed no dependency.
- `git diff --check`: clean. MDD state validation: `{"valid":true,"issues":[]}`; state was not modified.

## Changed paths

| Path | Change |
| --- | --- |
| `src/records/mod.rs` | Removes the custom recursive symlink resolver; opens the caller path and canonicalizes the now-existing database once for effective auxiliary accounting. No other production behavior changed. |
| `tests/records.rs` | Adds the test-only I/O fault child, concrete commit-failure, postcommit-over-limit and postcommit-unmeasured regressions, plus new-file and dangling-symlink characterization. |
| `tests/records_demo.rs` | Current nonvisual capture → save → remove origins → fresh-process read and quota-refusal journey. |
| `tests/records_measure.rs` | Frozen real-data resource protocol with capture, open, save/commit, read, reopen, storage and tree-RSS observations. |
| `tests/support/records/io_fault.c` | Test-only Linux `LD_PRELOAD` WAL write/fsync interposer, built by the test with the existing compiler. |
| `README.md` | Describes the durable-capture library that now exists, its limits, and keeps the CLI honest. |
| `docs/IMPLEMENTATION_ROADMAP.md` | Updates the existing AS-M004/L04 status line; no plan rewrite. |
| `.missions/AS-M004/evidence/P03/` | Qualification, measurements and demonstration evidence. |

## Path resolution simplification and characterization

Before changing the resolver, three characterizations ran on the P02C1 code and
passed: `path-alias-characterization.txt` (file and directory aliases),
`path-new-file-characterization.txt` (new database through a symlinked parent)
and `path-dangling-characterization.txt` (dangling database symlink). They
establish the preserved behavior; this was a refactor, not a behavior change, so
no RED is claimed.

`Store::open_with_budget` now opens the caller path (SQLite/OS resolve and create
it), then canonicalizes the actual existing database once and uses that identity
for `-wal`/`-shm`/`-journal` measurement. The 100+ line recursive resolver was
deleted. All 37 records tests stay green; alias admission still measures the
canonical footprint and postcommit usage still equals it.

## Failure qualification

The test-only interposer is enabled after a baseline commit and faults only the
next attempted capture. It is not linked into the library and adds no production
API.

- **Commit WAL write failure:** fails the next `-wal` `pwrite`/`write` with `EIO`.
  The child reports `Capture store is unavailable: disk I/O error`; the baseline
  remains readable with unchanged row counts and the attempted capture is absent.
  `fault-commit.txt`.
- **Postcommit measurement failure:** renames the main database after a WAL
  `fdatasync`. The child receives `CommittedUnmeasured` naming `Capture 2 was
  committed`; the parent restores the file, reopens the store and reads the
  committed record. `fault-postcommit.txt`.
- **Postcommit over-limit:** adds a 2 MiB auxiliary file after commit under a
  1.5 MB budget. The returned result is
  `saved:2:over=true:retained=2212328`; the record is readable and is not
  deleted. `fault-postcommit.txt`.

What this proves: SQLite call-boundary write failure and honest postcommit
outcomes in a fresh process. What it does not prove: device/power-loss
behaviour, a lying `fsync`, or a live filesystem read-only transition; the
interposer is not a physical fault model. The existing pre-commit lock-wait kill
and postcommit/preack ambiguity test remain separate. Details in
`failure-notes.md`.

## Resource protocol

`resource-protocol.md` freezes fixtures, host/load/cache conditions, run counts,
percentiles, raw observations, storage/auxiliary peaks and tree memory; raw
output is `measure-final.txt`. Final-run summary:

| Journey | p50 | p95 |
| --- | ---: | ---: |
| one-file capture only (30) | 61,448 µs | 69,155 µs |
| five-file capture only (30) | 91,686 µs | 110,034 µs |
| one-file save + synchronous commit (20) | 409 µs | 545 µs |
| five-file save + synchronous commit (20) | 768 µs | 883 µs |
| one-file read (20) | 231 µs | 288 µs |
| five-file read (20) | 405 µs | 450 µs |
| one-file capture + save + read (20) | 65,804 µs | 76,295 µs |
| five-file capture + save + read (20) | 97,756 µs | 110,277 µs |
| reopen existing store (20) | 276 µs | 358 µs |
| large-batch save + commit (1) | 145,470 µs | — |
| large-batch capture + save (1) | 651,817 µs | — |

Retained bytes: one-file 90,456; five-file 102,816; metadata-heavy (480 files,
1,146 payload) 181,096; 1 MiB single 1,149,296; 8 MiB batch 16,970,168;
held-reader two-save 3,316,416. Tree RSS peaks sampled 21–32 MB; final process
`VmHWM` 31,372 KiB and `VmRSS` 20,476 KiB.

Limits: storage and tree sampling is 1 ms, so sub-millisecond peaks and very
short Git children can be missed; recorded peaks are lower bounds. RSS includes
the test harness. OS caches were warm; this is not cold-disk measurement. One
host/load snapshot and small run counts do not establish future product budgets.
The admission formula remains a conservative engineering allowance under the
bundled SQLite/runtime assumptions, not a hard quota.

## Documentation and demonstration

README's current status and layout now describe `records` as available, with a
library example, integrity/commit/admission behavior, limits and the retained
test command. The roadmap status line now records L04 implementation under
qualification while its mission state owns acceptance. The CLI remains a
read-only configuration inspector; no records/analysis command was added.

`demonstration-intake.md` contains the one current nonvisual journey and exact
sanitized output (`demonstration-output.txt`): select `a.ts`, retain `guide.ts`
as context, save, refuse a zero-budget second capture while the first stays
readable, delete the repository, and read exact bytes/roles/generations in a
fresh process. It states the dirty base/head mapping, warm-cache limits and the
absence of a hosted trial or visual surface.

## Checks

`bash scripts/ci/check.sh` was run after stabilization with
`CI_BASE_SHA=45dac3c...` and passed (`ci-check.txt`): formatting, CI unit tests
(19), whitespace policy, workspace build, all-target tests, doctests, Clippy
with `-D warnings`, schema regeneration check and Node editor-service
conformance.

Focused evidence:

- `records-all.txt`: 37 records tests.
- `demonstration-output.txt`: 2 demo tests.
- `measure-final.txt`: resource protocol.
- `clippy-01.txt`, `build.txt`, `format-check.txt`: pass.
- `mdd-state-validation.txt`: valid.

## Remaining limits and gaps

- Admission is application policy, not a hard filesystem quota; custom SQLite
  builds with reserved page space or different checkpoint behaviour need
  re-qualification. Peak sampling is a lower bound.
- No resident/analyzer/model/report resource qualification, hosted trial,
  publication or final independent review was performed.
- Schema v1 is unshipped and has no migration; purge/export and future analysis
  remain later outcomes. The mission is not accepted by this report.
