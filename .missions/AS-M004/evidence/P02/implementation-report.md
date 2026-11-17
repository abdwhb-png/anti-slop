# AS-M004 P02 implementation report

done: candidate ready for examination

This is a bounded correction candidate only. It does not accept the packet, change mission state, commit, publish, merge or authorize No-Mistakes.

## Identity and context

- Role: implementer for packet `AS-M004-P02` revision 1, mission `AS-M004` revision 2.
- Execution identity: `as-m004-durable:s1794906814.562141.27268`, continued OpenCode session, provider route `opencode-go` / `deepseek-v4.1-flash`.
- High reasoning intent is recorded; this route exposes no verified selectable effort variant. Provider quota and actual inference identity remain unverified.
- MDD state validation before and after: `{"valid":true,"issues":[]}`. State was not modified.
- Accepted P01C1 source and evidence are preserved; all new proof is under `.missions/AS-M004/evidence/P02/`.

## Git state

- Base and HEAD unchanged: `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`). No commit, stash, reset, clean or dependency change was performed.
- Final status:

```text
 M Cargo.lock
 M Cargo.toml
 M src/lib.rs
?? .missions/AS-M004/
?? docs/missions/AS-M004/
?? src/records/mod.rs
?? tests/records.rs
?? tests/support/records/
```

- `Cargo.toml` and `Cargo.lock` retain the accepted P01/P01C1 state: one normal
  `rusqlite 0.41.1` declaration and the same nine qualified packages. P02 added
  no dependency or version and no Cargo file changed in this packet.
- `git diff --check`: clean. `src/config/` and `src/workspace/` are unchanged.

## Changed paths

| Path | Change |
| --- | --- |
| `src/records/mod.rs` | Adds `Budget` (10 GiB default), `StorageUsage`, `Store::{open_with_budget, usage}`, `BudgetExceeded`, actual database/auxiliary measurement, the conservative reservation formula, and admission inside the existing write transaction. |
| `tests/records.rs` | Adds twelve admission/storage/race/interruption tests and the child budget contender mode; preserves all 18 prior tests. |
| `.missions/AS-M004/evidence/P02/` | RED/GREEN logs, measurements, formula, command evidence. |

No schema change was needed: the budget is an explicit per-store input and is not persisted. The unshipped v1 schema remains as accepted in P01C1.

## Public interface and rationale

```text
Budget::new(limit) / Budget::default() -> 10 GiB
Store::open(path)                  // default budget
Store::open_with_budget(path, Budget)
Store::usage() -> StorageUsage { retained, limit, warning }
Store::save_capture(&Capture) -> Result<CaptureId, StoreError>
```

- `Budget` is explicit and caller-owned; it is not persisted, so reopening with
  a different policy is allowed. Opening never refuses on budget: a store whose
  retained data already exceeds the limit stays readable, and only new
  collection can be refused.
- `usage()` reports the actual sum of the main database plus existing `-wal`,
  `-shm` and `-journal` files. `warning` is true when retained bytes are at or
  above 80% of the limit (exact integer comparison, no float rounding).
- `save_capture` returns the existing `CaptureId` only after commit. A refusal
  is `StoreError::BudgetExceeded { retained, limit, required }`, with
  `required = retained + reservation`; callers see why and how much capacity the
  capture needs.
- The budget is application admission, not a filesystem quota. No parallel JSON
  parser, persistent reservation ledger, background process, lock framework or
  future API was added.
- `Store::open_with_budget` resolves the path lexically to an absolute path so
  later measurement is independent of the process working directory; it does
  not resolve symlinks into a different file identity.

## Admission and serialization

The order inside `save_capture` is:

1. `BEGIN IMMEDIATE` acquires the write transaction.
2. Read the actual retained footprint and compute the capture's reservation.
3. If `retained + reservation > limit`, return `BudgetExceeded` and roll back
   without inserting anything.
4. Build the capture rows and integrity digest, insert, and commit in the same
   transaction.

The transaction itself is the reservation lifetime. A second writer waits for
the first writer's commit (2 s busy bound) and then measures the committed WAL
and database, so two independent processes cannot both admit against the same
remaining capacity. There is no durable reservation row to leak: a process
killed before commit leaves no record, and the next writer can admit the same
capture under the same limit. A writer that cannot acquire the lock reports
`Busy` rather than admitting.

## Reservation formula

```text
payload     = fixed metadata + Σ(change path + 256)
            + Σ(source path + generation + 512 + source bytes)
            + configuration bytes
overflow    = ceil(payload / (page_size - 4))
row_pages   = (changes + sources + 1) * 8
pages       = overflow + row_pages + 16
wal_bound   = pages * (page_size + 24) + 32
reservation = 2 * wal_bound + current_wal_file_size + 64 KiB
required    = retained + reservation
```

Assumptions and evidence:

- WAL mode, `synchronous = FULL`; each frame carries a 24-byte header after the
  32-byte WAL header. Official SQLite file-format and WAL documents are cited in
  `admission-formula.md`.
- Page size is read from `PRAGMA page_size` (4096 here); the bundled build has
  zero reserved bytes, so an overflow page carries `page_size - 4` payload.
- All payload is charged as overflow pages, every row gets eight b-tree pages,
  and 16 pages cover header/free-page/rounding effects. This is deliberately
  conservative rather than a best-case packing estimate.
- The WAL bound is doubled because a checkpoint can leave the same committed
  content in both WAL and database. The existing WAL file size is added because
  a checkpoint can also backfill already committed frames into the database
  while the file remains. 64 KiB covers `-shm`, WAL headers and rounding.
- `journal_size_limit` is not used as a peak bound. A sum of source bytes or a
  `max_page_count` check would ignore page, metadata, WAL and checkpoint costs.

## Measured facts

Near-boundary fixture: eight untracked files of exactly 1,048,576 bytes plus the
186-byte committed configuration; 8,388,794 retained payload bytes.

```text
retained before admission   65,736
current WAL at admission    45,352
overflow pages               2,051
row pages (17 rows)            136
paged bound                  2,203
WAL bound                9,076,392
reservation             18,263,672
required                18,329,408
final retained          16,941,328
sampled peak            16,752,912   (140 samples at 1 ms)
```

The final database-plus-WAL size was about twice the source payload because the
data existed in both the checkpointed database and the recycled WAL. A
source-length-only account would have under-admitted this capture. Both the
sampled peak and final size stayed below `required`.

Repeated writes in one open store moved retained usage from 1,149,296 to
2,232,856 to 3,316,416 bytes, then jumped to 8,635,288 at the auto-checkpoint
near the 1000-page threshold; the next two writes reused the WAL file and
stayed at 8,635,288. All six records remained readable. A separate test showed
the reservation grows by exactly the measured WAL backlog when committed frames
have not yet been checkpointed.

## TDD evidence

The compiling scaffold (`Budget`, `open_with_budget`, `usage`, error variant)
returned zero usage, `warning: false` and no admission check. Then:

- `red-01-admission-absent.txt`: six behavioral failures — zero usage, false
  warning at exactly 80%, save succeeding under a 1-byte budget, no refusal at
  the boundary, no source-length distinction and two processes both saving.
- Measurement implemented: `retained_usage_counts_database_and_auxiliary_files`
  GREEN while the warning test stayed RED.
- Warning implemented: the 80% test GREEN.
- Admission, serialization and the backlog term implemented: refusal, exact
  boundary, source-length, race, checkpoint, peak and interruption tests GREEN.

No import/compiler failure was counted as RED and no working behavior was
mutated to manufacture one.

## Command results

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --frozen --test records -- --nocapture` | 30 passed | `green-05-final.txt` |
| `cargo test --frozen` | all targets passed: CLI 11, config 19, output 3, records 30, schema 3, workspace 34 (2 pre-existing ignored) | `full-tests.txt` |
| `cargo build --frozen` | success | `build.txt` |
| `cargo fmt --all -- --check` | success | `format-check.txt` |
| `cargo clippy --frozen --all-targets -- -D warnings` | success | `clippy-final.txt` |
| `git diff --check` | success | — |
| `node scripts/validate.mjs .../state.json --json` | `{"valid":true,"issues":[]}` | `mdd-state-validation.txt` |

## Race, failure and boundary outcomes

- Two independent process children opened the same store with a limit that
  admits one capture; exactly one saved and the other returned a budget refusal
  naming the limit. The baseline capture remained readable and only two capture
  rows exist.
- A save refused by the budget inserted no capture/change/source row; prior
  records remained readable.
- A store opened with a 1-byte budget still read the retained record; only the
  new save was refused.
- A writer killed while blocked before commit left no row and did not consume
  capacity: the same capture was admitted under the same limit afterward, and a
  following capture was refused once the limit was consumed.
- Replacing the database path with a directory made accounting return
  `Unavailable`; the save was refused rather than silently unbounded.
- A capture larger than the remaining capacity returns `BudgetExceeded` with
  the retained, limit and required values, before any insert.

## Limits and unrun checks

- The formula is conservative application policy, not a hard filesystem quota;
  it does not reserve disk blocks, detect a full device in advance, or account
  for other processes' files.
- The peak monitor samples file metadata every 1 ms; an instantaneous peak
  between samples is not observed, so measured peaks are lower bounds.
- Other readers can checkpoint concurrently; the current WAL size is added to
  cover backfill, but a different path spelling/symlink to the same database is
  not guaranteed to share accounting. Callers should use one path per store.
- If the formula were to under-estimate, the committed record is not deleted;
  `usage()` would report over-limit state and further saves would refuse. No
  automatic correction or deletion exists.
- Page-size/reserved-space assumptions match the bundled default build; a
  custom reserved-space build would need the usable-size assumption reviewed.
- Temporary SQLite files outside the workspace are not counted; the current
  statements do not create workspace-local temp databases.
- `scripts/ci/check.sh` was not run. The complete commit-interruption matrix,
  full journey latency/memory protocol, stabilized CI suite, current-status
  documentation and independent Codex whole-mission review remain open.

## Remaining mission obligations

Admission is now implemented and measured, but the mission still requires the
later full resource/latency and interruption qualification, stabilized CI
verification and the independent final review. Publication and merge remain
separately authorized.
