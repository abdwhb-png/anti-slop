# AS-M004 P01 evidence — exact durable capture round trip

- Mission: `AS-M004` revision 2, coordinated execution.
- Packet: `AS-M004-P01` revision 1.
- Base Git revision: `45dac3c241338c94166369fe821fe1d3499cf974` (the worktree HEAD at start and end; nothing was committed).
- Implementer context: fresh OpenCode worker session `as-m004-durable:s1794906814.562141.27268`, provider route `opencode-go` / model `deepseek-v4.1-flash`. High reasoning intent is recorded for the assignment; this route exposes no selectable effort variant. Actual provider quota and inference identity remain unverified by the worker.
- Worktree: `/home/abdwhb/engineering/firstmate-orchestration-test/data/as-m004-durable/worktree`.
- Date: 2026-11-17.

This is an implementation candidate only. It records no acceptance, checkpoint, mission advancement or delivery.

## Result

`anti_slop::records` now retains a complete workspace capture in SQLite and reads the exact bytes and provenance back in a fresh process after the source repository and configuration file are gone. Saves are single transactions; a returned `CaptureId` follows a completed commit. Schema initialization, reopen, newer-version refusal, unrelated or corrupt database refusal, explicit write failure, writer contention and interruption before and after commit are exercised through the real capture, storage and process boundaries.

## Scope and changed files

Tracked changes:

| Path | Change |
| --- | --- |
| `Cargo.toml` | Exact `rusqlite = "=0.41.1"` dependency with `bundled`, default features off, plus the same package as a dev-dependency for test-side lock and failure injection. |
| `Cargo.lock` | Nine added registry packages; see `dependency-qualification.md`. |
| `src/lib.rs` | Exports `records`; the module note no longer claims storage is absent. |
| `src/records/mod.rs` | New owning persistence module (schema, save, retained read, errors, locking). |

New test material: `tests/records.rs` and `tests/support/records/mod.rs`.

No change was needed in `src/workspace/` or `src/config/`. The existing public capture boundary already exposes everything the record needs (`Capture::{selection, resolved_commit, generation, changes, context, configuration, stats, slice}` plus `Source` fields, `Configuration::{generation, source, captured_bytes}` and `config::Source`). Reimplementing acquisition, selector or configuration semantics in `records` would have duplicated an existing owner, so the packet's minimal-change allowance was not used.

The coordinator-owned `.missions/AS-M004/` and `docs/missions/AS-M004/` files were left untouched. This packet added only `evidence/P01/`.

## RED → GREEN → REFACTOR

1. `red-01-missing-module.txt`: after writing the public-boundary tests and before adding the module, the target failed with `unresolved import anti_slop::records`. The same run also exposed a fixture-side `&mut Child` signature mistake; that test-harness defect was corrected before implementation and is not counted as RED.
2. `red-02-capability-absent.txt`: a clean repeat with only `pub mod records;` removed (fixture already correct) failed solely with the two intended unresolved-import errors.
3. `green-attempt-01.txt`: first implementation run, 10/10 tests pass. No assertion was weakened.
4. `green-03-records-final.txt`: final code (including `AUTOINCREMENT` and the additional identity assertions), 10/10 pass.
5. Mutation sensitivity checks, each reverted immediately:
   - `mutation-01-bytes-not-retained.txt`: binding empty bytes instead of retained bytes fails both byte assertions and the fresh-process range lookup.
   - `mutation-02-range-ignores-generation.txt`: dropping the generation check makes the unbound-range assertion fail.
   - `mutation-03-range-ignores-role.txt`: dropping the role check makes the wrong-role assertion fail.
   - `mutation-04-ignored-write-error.txt`: ignoring the source-insert result makes the injected-failure test fail because a failed save reports success.

The mutation checks show the assertions detect the behavior they claim to cover rather than passing vacuously.

## Tests and what each one catches

`tests/records.rs`:

- `retained_capture_survives_restart_and_loss_of_origins`: capture `Local` with distinct selected bytes and context bytes, an unchanged context file and a real committed configuration; save; drop the store; delete the whole repository; read in a separate process and compare id, selection, resolved commit, capture generation, configuration generation/sha256/captured bytes, source path/role/generation/blob/mode/bytes, and a byte range. Expected bytes and the configuration SHA-256 are computed independently of the retained row.
- `retained_content_preserves_deletion_empty_and_equal_occurrences`: a staged deletion, an empty file and two equal-content additions; the empty after-side stays present with zero bytes, the deletion after-side stays absent, and the equal additions keep separate path/generation identities. An empty retained range resolves.
- `empty_selection_round_trips_with_its_identities`: a valid empty capture, a `Base("HEAD")` capture (reference round trip) and a capture without a configuration file (`config::Source::BuiltIn`).
- `reopen_is_stable_and_incompatible_stores_are_rejected`: unchanged reopen, WAL persistence, missing id, unbound generation/role/path ranges, unsupported `user_version = 2`, an unrelated SQLite database, a non-database file and an unavailable parent directory.
- `failed_schema_initialization_leaves_no_partial_store`: a pre-existing view named `captures` makes schema creation fail inside its transaction; `user_version` stays 0 and no table is left behind.
- `injected_write_failure_leaves_no_partial_capture`: a test-created trigger aborts the second source insert; the save reports failure, row counts stay unchanged, the previous capture still reads, and a later save succeeds after the trigger is removed.
- `writer_contention_is_bounded_and_reported`: a second connection holds a write transaction; the save waits approximately `BUSY_TIMEOUT`, reports `Busy`, stores nothing, then succeeds after release.
- `committed_capture_survives_lost_acknowledgement`: a child writer commits and records its id, then is killed before its acknowledgement; the record is fully readable. Lost acknowledgement therefore does not prove absence.
- `writer_interrupted_before_commit_leaves_an_absent_capture`: a child writer reaches its save after the parent holds the write lock, is killed while the save waits, and leaves no capture, change or source row; a later save succeeds.
- `child_process_probe`: the exact-name fresh-process helper selected by the parent tests; it is inert without `AS_RECORDS_PROBE`.

## Checks run

All commands ran from the worktree root with `PATH=$HOME/.cargo/bin:$PATH`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo test --frozen --test records` | 10 passed | `green-03-records-final.txt` |
| `cargo test --frozen --test config --test schema --test workspace --test records` | 19 + 3 + 34 (2 ignored) + 10 passed | `tests-non-node-final.txt` |
| `cargo build --frozen` | success | part of `full-tests.txt` |
| `cargo fmt --all -- --check` | success | rerun in the final check |
| `cargo clippy --frozen --all-targets -- -D warnings` | success | `clippy.txt` |
| `python3 scripts/ci/policy.py dependencies --base 45dac3c241338c94166369fe821fe1d3499cf974` | 9 packages qualified | `policy-dependencies.txt` |

## Checks not run, and why

- `cargo test --frozen` cannot complete in this environment: it stops with 4 failures in `tests/cli.rs`, and a run that skips that target stops with 3 failures in `tests/output.rs`. Both fail only because `tools/config-conformance/node_modules` is absent: Node reports `ERR_MODULE_NOT_FOUND` for `@toon-format/toon`. These targets exercise TOON/default CLI conformance unrelated to this packet. The logs are `full-tests.txt` (stops at `cli`), `tests-without-cli.txt` and `tests-without-node-conformance.txt` (stop at `output`), and `tests-non-node-final.txt` (every runnable target passes). Restoring those packages (`sfw npm ci` in `tools/config-conformance`) was outside this packet's command grant, so it was not performed.
- `bash scripts/ci/check.sh` was not run because it includes the same editor-service conformance step and schema regeneration; the remaining Rust checks above were run individually.
- No measured latency, database/WAL peak growth, memory or admission-budget observation was made. The packet defers measured resource qualification, workspace budget admission, concurrent reservations and the complete crash-injection matrix to later work.
- No commit, push, PR, merge, No-Mistakes run, cleanup, install or provider change was performed.

## Known limits of this candidate

- Storage admission is not implemented: there is no budget, warning threshold, reservation or refusal at the configured collection threshold. This branch must not be presented as a released collection feature.
- The store opens read-write only; a genuinely read-only medium fails explicitly instead of serving retained reads.
- No purge/delete API exists, so `AUTOINCREMENT` identities cannot currently be reused; a future purge must preserve that guarantee.
- Schema version 1 has no migration path and no earlier supported schema exists, so the packet deliberately adds none.
- Interruption exactly inside SQLite's commit critical section is not deterministically injected; the complete crash matrix remains later work. The two exercised boundaries are "blocked before commit" and "killed after commit, before acknowledgement".
- A save retried after a lost acknowledgement creates a second record (each save is its own occurrence). Identity is not deduplicated, by design.
- The record retains capture inputs and configuration provenance, not analysis facts, analyzer/model identities, policy outcomes or reports.

## Remaining mission obligations

Workspace budget admission and warning/refusal boundaries, concurrent reservations that cannot both admit the same budget, the complete crash-injection matrix, measured database/journal/buffer cost and public-boundary journey timings, and the final independent whole-mission review remain open. Publication and merge remain separately authorized actions.

Supporting detail: `design.md`, `durability.md`, `dependency-qualification.md`.
