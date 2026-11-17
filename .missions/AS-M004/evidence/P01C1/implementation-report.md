# AS-M004 P01C1 implementation report

done: candidate ready for examination

This is a correction candidate only. It does not accept the packet, advance mission state, commit, publish, merge or authorize No-Mistakes.

## Identity and context

- Role: implementer for packet `AS-M004-P01C1` revision 1, mission `AS-M004` revision 2.
- Execution identity: `as-m004-durable:s1794906814.562141.27268`, continued OpenCode session, provider route `opencode-go` / `deepseek-v4.1-flash`.
- High reasoning intent is recorded; this route exposes no verified selectable effort variant.
- Provider quota and actual inference identity remain unverified.
- MDD state validation before and after: `{"valid":true,"issues":[]}`. State was not modified.
- The P01 candidate, its evidence and Firstmate's review/reproduction files were retained unchanged; all new evidence is under `.missions/AS-M004/evidence/P01C1/`.

## Git state

- Base and HEAD unchanged: `45dac3c241338c94166369fe821fe1d3499cf974` (`fm/as-m004-durable`). No commit, stash, reset or clean was performed.
- Retained uncommitted scope from P01 is preserved. Final status:

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

- `git diff --stat`: `Cargo.lock +71`, `Cargo.toml +1`, `src/lib.rs +3/-1`. The removed P01 dev-dependency leaves only the single qualified `rusqlite` declaration; `Cargo.lock` still contains exactly the same nine added packages and no new package or version.
- `git diff --check`: clean.

## Changes

| Path | Change |
| --- | --- |
| `src/records/mod.rs` | Adds the records-owned integrity digest, writes it in the capture transaction, verifies it before returning a read, rejects duplicate before/after evidence and unknown membership, and completes schema v1 with `captures.integrity` plus an open-time column check. |
| `tests/records.rs` | Adds eight public `Store` corruption regressions (missing source, missing change, altered bytes, altered provenance, duplicate source, extra change, altered capture provenance, altered configuration bytes) on top of the ten preserved tests. |
| `Cargo.toml` | Removes the redundant `rusqlite` dev-dependency; integration tests use the normal dependency. |
| `Cargo.lock` | Unchanged content; same nine qualified packages. |
| `src/lib.rs` | Unchanged from P01 (exports `records`). |

No `src/config/` or `src/workspace/` change was made or needed. Workspace and configuration identity algorithms remain solely owned where they already live.

## Actual RED assertion

The eight corruption tests were written and run before any production change. Each saves a real `Local` capture through `Store`, proves the record reads back intact (bytes `after\n`, context `guide\n`), applies one plausible mutation through a second SQLite connection, reopens the store and reads again. On the unchanged implementation all failed at the assertion that a corrupted record must not be returned as complete:

```text
panicked at tests/records.rs: corrupt retained evidence was returned as a complete capture
```

`red-01-corruption-undetected.txt` retains the seven-case run executed at that
point. An eighth case (missing change row) was added afterwards as a retained
regression; on the pre-correction code that state already produced an explicit
unknown-ordinal error rather than a successful read, so it is not claimed as
RED. No import error, fixture error or production mutation was used as RED.

## GREEN and regressions

- `green-01-corruption-detected.txt`: all seven corruption cases now return `StoreError::Corrupt`.
- `mutation-01-integrity-comparison-disabled.txt`: temporarily disabling the digest comparison makes six of the eight corruption regressions fail (missing and altered evidence); the duplicate and missing-change cases still fail explicitly from their structural checks, showing both layers are load-bearing. The mutation was reverted immediately.
- `green-02-records-all.txt`: 18/18 records tests pass. The preserved ten cover the full capture/save/reopen/fresh-process read, deletion versus empty, equal-content occurrences, valid empty and `Base` selections, built-in configuration, unsupported/unrelated/corrupt/unavailable stores, atomic initialization failure, injected rollback, bounded contention, post-commit kill and pre-commit kill. The eight new cases cover missing/altered/extra/duplicate evidence.
- `tests-non-node.txt`: config 19, records 18, schema 3, workspace 34 passed (2 pre-existing ignored).
- `node-conformance-tests.txt`: restored `tests/cli.rs` 11/11 and `tests/output.rs` 3/3 pass.
- `full-cargo-tests.txt`: complete `cargo test --frozen` passes every target.

## Integrity rationale

A save computes a domain-separated SHA-256 over a canonical view of the complete
retained record: comparison and configuration identity, change ordinals and
paths, and every source's ordinal, change membership, side, path, role,
generation, Git blob, Git mode, byte length and exact bytes. Optional values use
explicit presence markers and all fields are length-prefixed. The digest is
stored as `captures.integrity` in the same `BEGIN IMMEDIATE` transaction as the
rows and bytes; a failed transaction leaves no digest. A read reads the header
and all rows in one snapshot, recomputes the same digest from the stored values,
and returns `StoreError::Corrupt` on any mismatch before constructing a result.
It separately rejects unknown change ordinals, inconsistent side/membership and
a second before/after source for one change.

The digest is an integrity check over storage content, not a recomputation of
workspace or configuration identities. Retained generation strings are hashed
as opaque evidence; no workspace/config algorithm was copied. It detects
missing, extra, duplicate, reordered or altered rows and bytes, including
same-length changes to source and configuration bytes. It does not protect
against rewriting the evidence and the digest together or replacing the
database with a self-consistent forgery; there is no keyed MAC. Whole-capture
deletion is `NotFound`, not an integrity verdict. It does not prove physical
media integrity or recover data. Deletion-versus-empty and valid empty captures
are preserved because presence and zero-length byte sequences are encoded
distinctly.

Schema version 1 was unshipped, so the column is completed in place with no
migration. A pre-correction v1 database is refused at open as
`NotACaptureStore` because the required column is absent.

## Dependency and environment restoration

- The redundant `rusqlite` dev-dependency was removed through Cargo metadata/manifest; integration tests use the normal dependency. The graph is unchanged: `rusqlite 0.41.1`, `libsqlite3-sys 0.39.1`, `cc 1.11.1` and the same qualified transitives.
- `python3 scripts/ci/policy.py dependencies` on the complete current graph qualified 101 locked registry packages before any download (`policy-complete-graph.txt`).
- `sfw npm ci --ignore-scripts --no-audit --no-fund` in `tools/config-conformance` restored exactly the locked seven Node packages (`npm-ci.txt`). No lockfile or manifest change; `node_modules` is ignored and absent from `git status`.
- Cache limitation: Socket Firewall scans network fetches, not already-cached tarballs; npm still enforces lockfile integrity. The cache was preserved, not cleared.

## Commands and evidence

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --frozen --test records corruption` (before production change) | 7 failed on the actual successful-read defect | `red-01-corruption-undetected.txt` |
| `cargo test --frozen --test records corruption` | 7 passed | `green-01-corruption-detected.txt` |
| `cargo test --frozen --test records` | 18 passed | `green-02-records-all.txt` |
| `cargo test --frozen --test config --test schema --test workspace --test records` | 19 + 3 + 34 (2 ignored) + 18 passed | `tests-non-node.txt` |
| `cargo test --frozen --test cli --test output` | 11 + 3 passed | `node-conformance-tests.txt` |
| `cargo test --frozen` | all targets passed | `full-cargo-tests.txt` |
| `cargo build --frozen` | success | `build.txt` |
| `cargo fmt --all -- --check` | success | `format-check.txt` |
| `cargo clippy --frozen --all-targets -- -D warnings` | success | `clippy.txt` |
| `python3 scripts/ci/policy.py dependencies` | 101 packages qualified | `policy-complete-graph.txt` |
| `sfw npm ci --ignore-scripts --no-audit --no-fund` | 7 packages added, no lock change | `npm-ci.txt` |
| `node scripts/validate.mjs .../state.json --json` | `{"valid":true,"issues":[]}` | `mdd-state-validation.txt` |
| `git diff --check` | clean | — |

## Unrun checks and limits

- `bash scripts/ci/check.sh` was not run, as the packet directs; full stabilized mission verification remains later work.
- No performance, memory, database/WAL growth, admission-budget or measurement work was done. Resource qualification remains later mission work.
- Budget admission, warning/refusal thresholds, concurrent reservations, the complete crash-injection matrix and the independent Codex whole-mission review remain open.
- Integrity limits are stated above: no protection against consistent evidence-plus-digest rewriting, no physical-media verification, no recovery, and no claim that the digest replaces workspace generation identity.

## Remaining mission obligations

Admission and reservation behavior, the complete crash matrix, measured resource qualification and the independent final review are still required. Publication and merge remain separately authorized.
