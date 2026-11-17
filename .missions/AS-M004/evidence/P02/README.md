# AS-M004 P02 evidence — bounded durable collection admission

- Mission `AS-M004` revision 2, packet `AS-M004-P02` revision 1, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`.
- Implementer context: continued OpenCode session `as-m004-durable:s1794906814.562141.27268`, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no provider-selectable effort variant.
- The accepted P01C1 source and evidence are preserved. All P02 evidence is in this directory.
- Candidate only: no acceptance, state change, commit, publication or merge.

## Outcome

`anti_slop::records` now enforces a workspace-local admission budget:

- `Budget` is an explicit per-store input with a 10 GiB default (`DEFAULT_BUDGET_BYTES`).
- `Store::open_with_budget` keeps a store open and readable even when retained
  data already exceeds the budget; `Store::open` uses the default.
- `Store::usage()` reports actual retained database plus `-wal`, `-shm` and
  `-journal` bytes and a warning flag at or above 80%.
- `save_capture` measures actual storage and a conservative per-capture
  reservation inside the same `BEGIN IMMEDIATE` transaction that commits the
  capture. A capture that does not fit returns
  `BudgetExceeded { retained, limit, required }` before any row is inserted.
- Serially: the write transaction is the reservation lifetime. A competing
  writer waits for the winner's commit and then measures the committed result;
  no persistent reservation ledger exists to leak after a crash.

The formula and measured peaks are in `admission-formula.md`.

## TDD staging and RED

The new API needed a compiling scaffold before behavior could be asserted. The
scaffold added `Budget`, `open_with_budget`, `StorageUsage`/`usage()` and the
`BudgetExceeded` variant while returning zero usage, `warning: false`, and no
admission check. `red-01-admission-absent.txt` is the behavioral RED on that
scaffold:

| Test | Failure |
| --- | --- |
| `retained_usage_counts_database_and_auxiliary_files` | reported 0 retained bytes |
| `warning_starts_at_the_eighty_percent_line` | warning false at exactly 80% |
| `budget_refusal_preserves_prior_records_and_reads` | save under budget 1 succeeded |
| `admission_boundary_matches_the_reported_requirement` | no refusal; got `Ok(CaptureId(2))` |
| `source_lengths_alone_do_not_qualify_admission` | no refusal; got `Ok(CaptureId(2))` |
| `two_processes_cannot_both_admit_one_capture` | no refusal; got `Ok(CaptureId(2))` |

The failures were behavioral assertions after the scaffold compiled, not import
or fixture failures. Measurement was implemented first and passed; the warning
test stayed RED until the warning was implemented; admission/reservation and
the cross-process test then went GREEN. The final code has no scaffold
placeholder.

## GREEN

- `green-05-final.txt`: 30/30 records tests, including all preserved P01/P01C1
  round-trip, corruption, atomicity, lock and interruption tests.
- `full-tests.txt`: complete `cargo test --frozen` passes every target
  (CLI 11, config 19, output 3, records 30, schema 3, workspace 34 with 2
  pre-existing ignored).
- `clippy-final.txt`, `build.txt`, `format-check.txt`: pass.
- `git diff --check`: pass.

Admission-specific checks:

- `retained_usage_counts_database_and_auxiliary_files`
- `warning_starts_at_the_eighty_percent_line`
- `budget_refusal_preserves_prior_records_and_reads`
- `admission_boundary_matches_the_reported_requirement`
- `source_lengths_alone_do_not_qualify_admission`
- `two_processes_cannot_both_admit_one_capture`
- `reservation_accounts_for_the_existing_checkpoint_backlog`
- `repeated_writes_and_checkpoints_remain_accounted`
- `large_capture_peak_stays_within_admitted_budget`
- `interrupted_admission_leaks_no_capacity`
- `unavailable_storage_accounting_blocks_admission`

Measured facts from the final run:

```text
payload=8388794 retained_before=65736 reservation=18263672
required=18329408 final=16941328 peak=16752912 samples=140
checkpoint usages: [1149296, 2232856, 3316416, 8635288, 8635288, 8635288]
```

The 8 MiB payload produced 16.9 MiB of final database-plus-WAL storage; the
sampled peak and final size both stayed below the reported requirement.

## Commands

All commands ran from the worktree root with `PATH=$HOME/.cargo/bin:$PATH`.

| Command | Result |
| --- | --- |
| `cargo test --frozen --test records -- --nocapture` | 30 passed (`green-05-final.txt`) |
| `cargo test --frozen` | all targets passed (`full-tests.txt`) |
| `cargo build --frozen` | success (`build.txt`) |
| `cargo fmt --all -- --check` | success (`format-check.txt`) |
| `cargo clippy --frozen --all-targets -- -D warnings` | success (`clippy-final.txt`) |
| `git diff --check` | success |
| `node scripts/validate.mjs .../state.json --json` | `{"valid":true,"issues":[]}` |

## Scope and dependency state

Writable changes are confined to `src/records/`, `tests/records.rs`,
`tests/support/records/` and this evidence directory. No `src/config/`,
`src/workspace/`, Cargo manifest/lock, contract, packet, state, history, result
or instruction file changed. The single qualified dependency graph from P01C1
is unchanged: `Cargo.toml` still adds only the normal `rusqlite` declaration
and `Cargo.lock` still contains exactly the same nine packages.

No commit, push, PR, merge, No-Mistakes run, installation, cleanup or reviewer
dispatch was performed. `scripts/ci/check.sh` was not run.

## Unrun and remaining mission work

The complete commit-interruption matrix, full journey latency/memory protocol,
stabilized CI shell suite, current-status documentation and the independent
Codex whole-mission review remain open. The peak monitor samples file metadata
at 1 ms intervals, so an instantaneous peak between samples is not observed;
the recorded peak is a lower bound. The admission bound is application policy,
not a hard filesystem quota.

## Files

- `red-01-admission-absent.txt`
- `green-01-admission.txt`, `green-02-admission-backlog.txt`,
  `green-03-checkpoints.txt`, `green-04-records-final.txt`, `green-05-final.txt`
- `full-tests.txt`, `clippy-01.txt`, `clippy-final.txt`, `build.txt`,
  `format-check.txt`
- `admission-formula.md`, `implementation-report.md`
- `git-status.txt`, `git-diffstat.txt`, `tracked-source.diff`,
  `Cargo.lock.diff`, `cargo-tree.txt`, `tool-versions.txt`,
  `mdd-state-validation.txt`
