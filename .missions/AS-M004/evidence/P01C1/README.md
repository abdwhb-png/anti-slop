# AS-M004 P01C1 evidence — retained-capture integrity correction

- Mission `AS-M004` revision 2, packet `AS-M004-P01C1` revision 1, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`.
- Implementer context: continued OpenCode worker session `as-m004-durable:s1794906814.562141.27268`, route `opencode-go` / `deepseek-v4.1-flash`; high intent with no provider-selectable effort variant.
- Date: 2026-11-17.
- This is a correction candidate only: no acceptance, state change, commit, publication or delivery.

The rejected P01 candidate and its evidence were preserved unchanged. New proof
lives only in this directory.

## Behavioral RED on the unchanged implementation

`red-01-corruption-undetected.txt` records seven public-`Store` regressions run
before any production change. Each test saves a real capture, proves it reads
back intact, applies one plausible damage through a second SQLite connection,
reopens the public store and reads again. All seven failed with:

```text
corrupted retained evidence was returned as a complete capture
```

Cases: missing after source; altered same-length source bytes; altered role and
generation; duplicated source row; extra change row; altered capture
generation; altered same-length configuration bytes. No import, fixture or
production mutation was involved; the RED is the actual successful-read defect.

A separate missing-change regression was added after the RED run. On the
pre-correction code that state already produced an explicit unknown-ordinal
error, so it is retained as a regression but is not counted as RED.

## GREEN

`green-01-corruption-detected.txt` — the seven integrity regressions pass.

`green-02-records-all.txt` — all 18 records tests pass, including the original
ten (round trip, deletion vs empty, empty/Base/built-in captures, reopen and
version refusal, atomic initialization, injected rollback, contention,
post-commit kill, pre-commit kill). Deletion-versus-empty and valid empty
captures are unchanged.

`tests-non-node.txt` — config, schema, workspace and records targets pass.

`node-conformance-tests.txt` — restored `tests/cli.rs` (11) and
`tests/output.rs` (3) pass.

`full-cargo-tests.txt` — complete `cargo test --frozen`: all targets pass.

## Node conformance restoration

Narrowly authorized restoration of the already locked Node packages, not a
dependency change:

1. `python3 scripts/ci/policy.py dependencies` on the complete current graph
   qualified 101 locked registry packages, including the seven npm packages
   (`policy-complete-graph.txt`). This ran before any download.
2. `sfw npm ci --ignore-scripts --no-audit --no-fund` in
   `tools/config-conformance` added seven packages (`npm-ci.txt`).
3. No `package.json`/`package-lock.json` change; `node_modules` is ignored by
   `tools/config-conformance/.gitignore` and does not appear in `git status`.
4. `sfw` version 1.16.0. Cache limitation: Socket Firewall scans network
   traffic, not already-cached tarballs; npm still verifies the lockfile
   integrity hashes. The cache was preserved, not cleared.

## Other checks

- `cargo build --frozen`: success (`build.txt`).
- `cargo fmt --all -- --check`: success (`format-check.txt`).
- `cargo clippy --frozen --all-targets -- -D warnings`: success (`clippy.txt`).
- `git diff --check`: success.
- No `bash scripts/ci/check.sh`, as directed; full stabilized mission
  verification remains later work.

## Integrity rationale and limits

See `integrity-notes.md`. In short: a records-owned domain-separated SHA-256
covers the complete retained membership, provenance and bytes; it is stored in
the same transaction and recomputed before any read result is returned. It
detects missing/altered/extra/duplicate evidence and same-length byte changes.
It does not resist a party that rewrites the evidence and its digest together,
does not turn whole-row deletion into an integrity report (`NotFound`), and
does not replace filesystem/device integrity or workspace-generation identity.

## Files in this directory

- `red-01-corruption-undetected.txt`, `green-01-corruption-detected.txt`, `green-02-records-all.txt`, `mutation-01-integrity-comparison-disabled.txt`.
- `tests-non-node.txt`, `node-conformance-tests.txt`, `full-cargo-tests.txt`.
- `policy-complete-graph.txt`, `npm-ci.txt`.
- `integrity-notes.md`, `implementation-report.md`.
- `build.txt`, `format-check.txt`, `clippy.txt`, `tool-versions.txt`.
- `git-status.txt`, `git-diffstat.txt`, `tracked-source.diff`, `Cargo.lock.diff`.
