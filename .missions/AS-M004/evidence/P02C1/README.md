# AS-M004 P02C1 evidence — effective database identity in quota accounting

- Mission `AS-M004` revision 2, packet `AS-M004-P02C1` revision 1, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`.
- Implementer context: continued OpenCode session `as-m004-durable:s1794906814.562141.27268`, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no provider-selectable effort variant.
- Rejected P02 evidence and all prior mission work are preserved. This directory holds P02C1 proof.
- Candidate only: no acceptance, state change, commit, publication or merge.

## Defect

P02 measured suffixes of the caller's path spelling. SQLite's Unix VFS resolves
symbolic links in every path element before deriving `-wal`/`-shm`, so an alias
opened a real database while `records` measured only the 4 KiB main file through
the alias. With a held read snapshot preserving the canonical WAL backlog, P02
admitted a write that exceeded the reported budget.

## RED on the unchanged P02 implementation

`red-01-alias-undercount.txt` — the public `Store` regression opened a real
database through a symlink while a reader held a snapshot:

```text
alias retained=4096 canonical_before=6459976
admitted=7016016 saved=Ok(CaptureId(3)) canonical_after=9657096
```

The alias admission saw only the 4 KiB main database file, omitted the canonical
`-wal`/`-shm`, accepted `7,016,016` bytes, and returned success while the
canonical files occupied `9,657,096`. This is the intended behavioral RED: a
successful public read/refusal path undercounted real storage.

A no-reader run (Firstmate's counterfactual, retained in
`firstmate-no-reader-counterfactual.json`) closes the last connection, checkpoints
and removes the auxiliaries, so the defect is masked. It is not treated as a
separate duplicate test.

## GREEN

`green-04-final-records.txt` — 31/31 records tests. The new
`quota_accounting_uses_the_effective_database_files` covers both a file symlink
and a parent-directory symlink. `green-03-alias-observations.txt` records:

```text
file alias: retained=9,657,096 canonical_before=9,657,096 required=26,289,248
            canonical_after=12,854,216 post_commit=12,854,216
dir  alias: retained=12,854,216 canonical_before=12,854,216 required=32,683,488
            canonical_after=16,051,336 post_commit=16,051,336
```

All previous admission, corruption, transaction, lock and interruption tests
remain green. `full-tests.txt` shows the complete `cargo test --frozen` passing
all targets (CLI 11, config 19, output 3, records 31, schema 3, workspace 34
with 2 pre-existing ignored). `build.txt`, `format-check.txt` and `clippy.txt`
pass.

## Code change

- `src/records/mod.rs`: resolves the effective database path before opening and
  measuring, so SQLite's real `-wal`/`-shm`/`-journal` files are counted even
  when callers use a file or directory symlink. Failed resolution is an explicit
  `Unavailable` error.
- `save_capture` now returns `SavedCapture { id, usage }`, measured after commit;
  `StorageUsage` has `over_limit`; a post-commit measurement failure returns
  `CommittedUnmeasured { id, reason }` instead of an ordinary failure. A
  committed over-budget footprint is visible and is never deleted.
- The reservation now adds dynamic selection-reference, commit, generation,
  configuration-path and hash lengths to the fixed header allowance.
- Opening/initialization remains allowed below a low budget, but schema/file
  bytes are included in later admission accounting.

No schema, Cargo, dependency, config, workspace or other source change was made.

## Commands

All commands ran from the worktree root with `PATH=$HOME/.cargo/bin:$PATH`.

| Command | Result |
| --- | --- |
| `cargo test --frozen --test records -- --nocapture` | 31 passed (`green-04-final-records.txt`) |
| `cargo test --frozen` | all targets passed (`full-tests.txt`) |
| `cargo build --frozen` | success (`build.txt`) |
| `cargo fmt --all -- --check` | success (`format-check.txt`) |
| `cargo clippy --frozen --all-targets -- -D warnings` | success (`clippy.txt`) |
| `git diff --check` | success |
| `node scripts/validate.mjs .../state.json --json` | `{"valid":true,"issues":[]}` |

## Remaining limits

The peak monitor samples at 1 ms, so peaks between samples are lower bounds.
The reservation remains a conservative engineering allowance validated by
measured peaks, not a formal storage bound for arbitrary filesystems or
custom SQLite builds with reserved page space. External non-workspace temp files
and unrelated files are not charged. The complete interruption matrix, journey
latency/memory protocol, stabilized CI shell suite, documentation and Codex
whole-mission review remain open.

No `scripts/ci/check.sh`, commit, dependency change, publication or reviewer
dispatch was performed.
