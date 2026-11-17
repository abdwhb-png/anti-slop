# Terminal report — AS-M004 P01

done: candidate ready for examination

This is an implementation candidate only. No acceptance, checkpoint, mission advancement, commit, publication or delivery is claimed.

## Identity and context

- Role: implementer (coordinated MDD execution).
- Assignment: packet `AS-M004-P01` revision 1, mission `AS-M004` revision 2, base `45dac3c241338c94166369fe821fe1d3499cf974`.
- Worktree: `/home/abdwhb/engineering/firstmate-orchestration-test/data/as-m004-durable/worktree`.
- Fresh worker context: `as-m004-durable:s1794906814.562141.27268`, launched through the stored `opencode-go` / `deepseek-v4.1-flash` route. No prior session conversation was used.
- Reasoning intent: high, as recorded for the assignment. This provider route exposes no selectable effort variant; that mismatch is a route limitation, not a downgrade chosen here.
- Provider quota and actual inference identity remain unverified. No provider, account or home configuration was changed.
- MDD state validation at session start and end: `{"valid":true,"issues":[]}` (`mdd-state-validation.txt`). State was not modified.
- Coordinator-owned preparation files under `.missions/AS-M004/` and `docs/missions/AS-M004/` were retained unchanged; this packet only added `.missions/AS-M004/evidence/P01/`.

## Git state

- Starting HEAD and ending HEAD: `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`). Nothing was committed, stashed, reset or cleaned.
- Worktree was clean at start apart from the coordinator's untracked `.missions/AS-M004/` and `docs/missions/AS-M004/`.
- Ending `git status --porcelain=v1`:

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

- `git diff --check`: clean.
- `src/config/` and `src/workspace/` are unchanged. No project instruction file, contract, packet, state, history or result file was edited.

## Changed files

| Path | Nature |
| --- | --- |
| `Cargo.toml` | Adds `rusqlite = { version = "=0.41.1", default-features = false, features = ["bundled"] }` and the same exact package as a dev-dependency for test-side lock/trigger inspection. |
| `Cargo.lock` | Nine added registry packages; `cc` pinned to `1.11.1` for the seven-day window. |
| `src/lib.rs` | Exports `records`; removes the stale statement that storage is unavailable. |
| `src/records/mod.rs` | New persistence owner: schema v1, open/initialize/verify, save, retained read, error mapping, bounded locking. |
| `tests/records.rs` | Public-boundary tests, including two fresh-process probes. |
| `tests/support/records/mod.rs` | Real Git repository fixtures and process helpers. |
| `.missions/AS-M004/evidence/P01/` | Compact proof: RED/GREEN, mutation checks, dependency policy, durability sources, design, tool versions, logs. |

No workspace/config accessor change was necessary: the existing capture already exposes selection, resolved commit, capture generation, change/context sources with path/role/generation/blob/mode/bytes, configuration generation/source/captured bytes and range slicing. Adding a parallel representation would have duplicated an existing owner.

## Interface and design rationale

`anti_slop::records` exposes:

```text
Store::open(&Path) -> Result<Store, StoreError>
Store::save_capture(&mut self, &Capture) -> Result<CaptureId, StoreError>
Store::read_capture(&self, CaptureId) -> Result<RetainedCapture, StoreError>
RetainedCapture::{id, selection, resolved_commit, generation, configuration, changes, context, slice}
```

- `records` consumes `workspace::Capture`; it does not re-run capture, selection or configuration resolution. It reuses `Role`, `GitMode`, `Selection`, `SourceRange` and `config::Source` rather than defining parallel enums.
- SQLite stays behind the boundary: `StoreError` distinguishes unavailable storage, unsupported newer schema, unrelated database, busy writer, damage, failed operation, missing record and unbound range; `rusqlite` types are not exposed.
- Schema v1 uses `PRAGMA user_version`, three tables (`captures`, `capture_changes`, `capture_sources`) and table checks for the valid selection/configuration/source states. A save is one `BEGIN IMMEDIATE` transaction; the capture row, every change and every source row (including exact BLOB bytes) commit together. `CaptureId` is returned only after `commit()` succeeds.
- `AUTOINCREMENT` ids keep save occurrences distinct and prevent later identity reuse. No content hash deduplication exists, so separate occurrences of equal bytes keep their own provenance.
- Retained reads run in one read snapshot and reconstruct changes/context, including the change path that differs from a rename's before path. `slice` binds path, role and generation, mirroring `Capture::slice`.
- Journal mode `WAL` and `synchronous = FULL` are set and read back on open; the busy timeout is a declared 2 s bound. `journal_size_limit` is intentionally not used as a peak bound.
- No repository/service wrapper, migration framework, background queue, external blob store, prepared-statement cache or speculative extension point was added.

## Dependency qualification

- Selected `rusqlite 0.41.1` (published 2026-10-28, not yanked, Rust 1.89) and resolved `libsqlite3-sys 0.39.1` (2026-10-25, Rust 1.88). `bundled` compiles SQLite 3.54.1. Default features are off to avoid the statement cache and wasm-only dependency.
- Rejected as inside the seven-day window: `rusqlite 0.43.0` and `0.42.0`, `libsqlite3-sys 0.40.0`, and `cc 1.11.2`. `cc` was pinned to `1.11.1`.
- `python3 scripts/ci/policy.py dependencies --base 45dac3c241338c94166369fe821fe1d3499cf974` qualifies all nine added packages, with live registry publication dates, yank status and locked checksums (`policy-dependencies.txt`). The first policy run refused the graph while `cc 1.11.2` was locked; that refusal is retained.
- All networked Cargo operations used `sfw` (1.16.0): `metadata`, `update -p cc --precise 1.11.1`, `fetch --locked`.

Disclosed deviation: `sfw cargo metadata --format-version 1` resolved and downloaded the nine archives in one step before `policy.py` ran. No build or test ran before qualification, and the final graph is the qualified one, but the intended resolve → policy → fetch order was not achieved. Cached archives were preserved rather than cleared.

## TDD evidence

- RED after the public-boundary tests and before the module: `unresolved import anti_slop::records` (`red-01-missing-module.txt`; that run also exposed a test-harness `&mut Child` mistake, corrected before implementation).
- Clean RED after the fixture fix, with only `pub mod records;` removed: two intended unresolved-import errors (`red-02-capability-absent.txt`).
- GREEN: 10/10 records tests (`green-attempt-01.txt`, final `green-03-records-final.txt`).
- Mutation sensitivity, each reverted immediately:
  - empty bytes instead of retained bytes → both byte assertions and the fresh-process range fail;
  - slice ignores generation → unbound-range assertion fails;
  - slice ignores role → wrong-role assertion fails;
  - ignored source-insert error → injected-failure test fails because a failed save would report success.

The mutation checks establish that the suite detects the behavior it claims to verify.

## Command outcomes

All commands ran from the worktree root with `PATH=$HOME/.cargo/bin:$PATH`.

| Command | Outcome |
| --- | --- |
| `cargo test --frozen --test records` | 10 passed, 0 failed (`green-03-records-final.txt`) |
| `cargo test --frozen --test config --test schema --test workspace --test records` | 19 + 3 + 34 (2 ignored) + 10 passed (`tests-non-node-final.txt`) |
| `cargo build --frozen` | success (`build.txt`) |
| `cargo fmt --all -- --check` | success (`format-check.txt`) |
| `cargo clippy --frozen --all-targets -- -D warnings` | success (`clippy.txt`) |
| `git diff --check` | success |
| `python3 scripts/ci/policy.py dependencies --base 45dac3c241338c94166369fe821fe1d3499cf974` | all nine packages qualified (`policy-dependencies.txt`) |
| `node .../scripts/validate.mjs .../.missions/AS-M004/state.json --json` | `{"valid":true,"issues":[]}` (`mdd-state-validation.txt`) |

## Failed and unrun checks

- `cargo test --frozen` cannot complete here: `tests/cli.rs` fails 4 tests and, when that target is skipped, `tests/output.rs` fails 3 tests. Every failure is `ERR_MODULE_NOT_FOUND` for `@toon-format/toon` because `tools/config-conformance/node_modules` is absent. These are TOON/default-conformance targets unrelated to this packet. Logs: `full-tests.txt`, `tests-without-cli.txt`, `tests-without-node-conformance.txt`; the runnable targets pass in `tests-non-node-final.txt`.
- `bash scripts/ci/check.sh` was not run because it includes that editor-service conformance step. Restoring the Node packages (`sfw npm ci` in `tools/config-conformance`) was outside this packet's command grant, so it was not done.
- No performance, latency, database/WAL peak growth, memory or admission-budget measurement was taken; the packet defers measured resource qualification.
- No commit, push, PR, merge, No-Mistakes run, cleanup, global install or provider/account change was performed.

## What the tests demonstrate

- Capture → save → drop store → delete the entire repository → fresh process read returns exact selected and context bytes, roles, Git blob/mode, source generations, capture/comparison/configuration identities, configuration bytes and a byte range. Expected bytes, the range slice and the configuration SHA-256 are checked independently.
- A deletion stays absent (`after: None`), an empty file stays present with zero bytes, and two equal-content additions keep distinct path/generation occurrences.
- A valid empty selection, a `Base("HEAD")` selection and a built-in configuration (no configuration file) round-trip.
- Unchanged reopen works; `user_version = 2` is refused as unsupported; an unrelated SQLite database, a non-database file and a missing parent are refused explicitly.
- A conflicting pre-existing schema object makes schema initialization fail and leaves version 0 with no tables.
- An injected source-insert failure rolls back the whole capture; prior rows remain readable and a later save succeeds.
- A competing writer causes a bounded wait then `Busy` with no row written; release allows the save.
- A writer killed after commit but before its acknowledgement leaves a fully readable record.
- A writer killed while waiting before commit leaves no capture/change/source row; recovery succeeds.

## Known limits

- Workspace budget admission, the 80% warning, threshold refusal, concurrent reservations and overshoot handling are **not implemented**. This branch is not a released collection feature.
- The store opens read-write; a genuinely read-only medium fails explicitly rather than serving retained reads.
- No purge API exists yet; a future purge must keep `AUTOINCREMENT` identity guarantees.
- Schema v1 has no migration path and no earlier supported schema exists; none was invented.
- Interruption exactly inside SQLite's commit critical section was not injected; the complete crash matrix remains later work. The exercised boundaries are blocked-before-commit and killed-after-commit-before-acknowledgement.
- A retry after a lost acknowledgement produces a second record; each save is a separate occurrence and identity is intentionally not deduplicated.
- The record retains capture inputs and configuration provenance only, not analyzer/model identities, policy outcomes, facts or reports.
- `journal_size_limit` is not set and would not bound peak WAL growth; no auxiliary growth measurement exists yet.

## Remaining mission obligations

Budget admission and refusal boundaries, concurrent reservation proof, the complete crash-injection matrix, reproducible database/WAL/buffer and journey measurements, and the independent Codex whole-mission review remain open. Publication, merge and acceptance require separate authority.

## Administrative note

One accidental empty placeholder file was created at `data/as-m004-durable/workflow-placeholder.md` while writing evidence and was removed immediately; it was outside the worktree, unrelated to the project, and left no content. No other write outside the worktree occurred except this report and the packet evidence.

## Evidence index

`.missions/AS-M004/evidence/P01/`:

- `README.md` — packet evidence summary and exact command log.
- `design.md` — interface, schema, failure/ambiguity and concurrency rationale.
- `durability.md` — journal/synchronous/busy choices with official SQLite citations and limits.
- `dependency-qualification.md` — registry facts, feature choice, policy result, ordering deviation.
- `red-01-missing-module.txt`, `red-02-capability-absent.txt`, `green-attempt-01.txt`, `green-03-records-final.txt`.
- `mutation-01-bytes-not-retained.txt`, `mutation-02-range-ignores-generation.txt`, `mutation-03-range-ignores-role.txt`, `mutation-04-ignored-write-error.txt`.
- `clippy.txt`, `format-check.txt`, `build.txt`, `full-tests.txt`, `tests-without-cli.txt`, `tests-without-node-conformance.txt`, `tests-non-node-final.txt`.
- `policy-dependencies.txt`, `dependency-qualification-first-policy-failure.txt`, `cargo-tree.txt`, `tool-versions.txt`.
- `mdd-state-validation.txt`, `git-status.txt`, `git-diffstat.txt`, `tracked-source.diff`, `Cargo.lock.diff`.
