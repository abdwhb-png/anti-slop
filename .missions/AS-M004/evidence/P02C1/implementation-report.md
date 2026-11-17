# AS-M004 P02C1 implementation report

done: candidate ready for examination

This is a bounded correction candidate only. It does not accept the packet, change mission state, commit, publish, merge or authorize No-Mistakes.

## Identity and Git provenance

- Role: implementer for packet `AS-M004-P02C1` revision 1, mission `AS-M004` revision 2.
- Execution identity: `as-m004-durable:s1794906814.562141.27268`, continued OpenCode session, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no verified selectable effort variant. Provider quota and inference identity remain unverified.
- Base and HEAD unchanged: `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`). No commit, stash, reset, clean or dependency change.
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

- `Cargo.toml`, `Cargo.lock` and `src/lib.rs` retain the accepted P01/P01C1/P02 state. P02C1 changed no Cargo file, no schema, no `src/config/`, no `src/workspace/`, no instruction/contract/packet/state/history/result and no prior evidence.
- MDD state validation before and after: `{"valid":true,"issues":[]}`. `git diff --check`: clean.

## Defect

P02 measured suffixes of the caller's spelling. SQLite's Unix VFS resolves
symbolic links in every path element before deriving auxiliaries, so opening
`alias.db` (a symlink to `real.db`) reads the real database while P02 measured
only the 4 KiB main file through the alias. With a held reader preserving the
canonical WAL backlog, admission undercounted by the entire `-wal`/`-shm`
footprint and accepted a write that exceeded the budget. Without the held
reader, last-close checkpoint removes the auxiliaries and masks the defect.

## Behavioral RED

`red-01-alias-undercount.txt`, public `Store` boundary on the unchanged P02
code, with a real reader transaction held open:

```text
alias retained=4096 canonical_before=6459976
admitted=7016016 saved=Ok(CaptureId(3)) canonical_after=9657096
```

The refusal measured only the main file, the next save was admitted at
`7,016,016`, and the committed canonical files occupied `9,657,096`. This is
the intended behavioral failure: a successful public path/accounting result
contradicted the actual database footprint. No compiler/import failure or
production mutation was counted as RED.

## Correction

`src/records/mod.rs` now resolves the effective database identity before
opening and measuring:

- existing files are resolved with `fs::canonicalize`, so file symlinks and
  parent-directory symlinks both name the real database;
- a not-yet-created database resolves its parent and keeps the final name;
- a dangling final symlink is followed (bounded) to the path SQLite would
  create;
- resolution failure is an explicit `Unavailable` error, never a smaller
  measurement.

The write transaction remains the writer-serialization and reservation owner.
No path registry, lock framework or multi-process ledger was added.

## Post-commit truth

`save_capture` now returns `SavedCapture { id, usage }`. The usage is measured
after the commit and carries `retained`, `limit`, `warning` and `over_limit`.
A successful caller therefore cannot receive a bare id while the committed
footprint silently differs. If the post-commit measurement itself fails, the
error is `CommittedUnmeasured { id, reason }`, which names the committed record
rather than claiming absence. No code deletes data or turns a committed write
into a clean failure.

The alias regression asserts the returned post-commit usage equals the canonical
footprint, the committed footprint is within the admitted requirement, and the
record is readable.

## GREEN

`green-04-final-records.txt` — 31/31 records tests, including all prior
corruption, round-trip, transaction, lock, interruption, warning, boundary,
checkpoint, peak and multi-process admission tests. The new
`quota_accounting_uses_the_effective_database_files` covers both alias forms.

Observed final alias runs:

```text
file alias: retained=9,657,096 canonical_before=9,657,096 required=26,289,248
            canonical_after=12,854,216 post_commit=12,854,216
dir  alias: retained=12,854,216 canonical_before=12,854,216 required=32,683,488
            canonical_after=16,051,336 post_commit=16,051,336
```

`full-tests.txt` — complete `cargo test --frozen`: CLI 11, config 19, output 3,
records 31, schema 3, workspace 34 (2 pre-existing ignored), all passing.
`build.txt`, `format-check.txt`, `clippy.txt` and `git diff --check` pass.

## Formula review

The reservation constants are justified against the actual bounded inputs and
the installed SQLite build in `formula-review.md`:

- Capture limits: 512 files, 1 MiB per input, 16 MiB total, 1 MiB configuration,
  4,096-byte normalized paths/base reference; generation 64, Git blob 40/64.
- `BASE_METADATA_BYTES = 4 KiB` covers fixed capture-row/page metadata; dynamic
  selection-reference, commit, generation, configuration-path and hash lengths
  are now added explicitly.
- `CHANGE_METADATA_BYTES = 256` and `SOURCE_METADATA_BYTES = 512` cover non-path
  row fields; eight b-tree pages per row is a conservative engineering allowance
  for the bounded row counts, not a formal arbitrary-storage proof.
- Page size is read at open; overflow payload is `page_size - 4`; the bundled
  default has zero reserved bytes.
- WAL frame/header sizes are official; the factor two covers checkpoint
  duplication; the current WAL file size covers backfill of committed frames;
  64 KiB covers `-shm` and rounding. `journal_size_limit` is not a peak bound.
- Supported conditions: Linux/WSL2, bundled SQLite 3.54.1, WAL,
  `synchronous = FULL`, default auto-checkpoint, one writer transaction, readers
  allowed. Custom reserved-space builds, unrelated workspace files and hard
  filesystem quotas are outside the guarantee.

Opening and initialization remain allowed below a low budget; retained reads
work, and the schema/file bytes are included in every later admission
measurement. The 10 GiB default, exact 80% warning and explicit refusal remain
unchanged.

## Commands

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --frozen --test records -- --nocapture` | 31 passed | `green-04-final-records.txt` |
| `cargo test --frozen` | all targets passed | `full-tests.txt` |
| `cargo build --frozen` | success | `build.txt` |
| `cargo fmt --all -- --check` | success | `format-check.txt` |
| `cargo clippy --frozen --all-targets -- -D warnings` | success | `clippy.txt` |
| `git diff --check` | clean | — |
| `node scripts/validate.mjs .../state.json --json` | `{"valid":true,"issues":[]}` | `mdd-state-validation.txt` |

## Unrun checks and remaining limits

- The post-commit `CommittedUnmeasured` measurement-failure branch is defensive
  and was not forced deterministically; the normal post-commit result is tested.
  The committed write remains the truth in that path.
- The peak monitor samples at 1 ms, so instantaneous peaks are lower bounds.
- The reservation is conservative application policy, not a hard quota; it does
  not account for unrelated workspace files or SQLite temp files outside the
  database directory, and custom reserved-space builds need re-qualification.
- `scripts/ci/check.sh` was not run. The complete commit-interruption matrix,
  journey latency/memory protocol, stabilized CI suite, current-status
  documentation and the independent Codex whole-mission review remain open.

No acceptance, state advance, publication, No-Mistakes run or merge was
performed. The candidate stops here for Firstmate examination.
