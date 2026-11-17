# AS-M004 P03 evidence — failure, resource and documentation qualification

- Mission `AS-M004` revision 2, packet `AS-M004-P03` revision 1, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`.
- Implementer context: continued OpenCode session `as-m004-durable:s1794906814.562141.27268`, route `opencode-go` / `deepseek-v4.1-flash`; high intent, no provider-selectable effort variant.
- All accepted P01/P01C1/P02/P02C1 work and rejected evidence remain preserved; the mission is uncommitted in this worktree.
- Candidate only: no acceptance, checkpoint, commit, publication, merge or No-Mistakes. Firstmate examines; Codex remains the independent final reviewer.

## Outcome

- Actual `-wal` write failure during a synchronous commit is exercised through a
  test-only Linux interposer; the failed capture is absent, prior records remain
  readable, and the error is explicit.
- The previously unforced `CommittedUnmeasured` and postcommit over-limit
  outcomes are exercised: the error names the committed id, and a successful
  over-limit save reports `over_limit=true` with its actual retained bytes.
- The custom recursive symlink resolver was removed. SQLite/OS now own path
  resolution: the store opens the caller path, then canonicalizes the actual
  existing database once for auxiliary accounting. File aliases, directory
  aliases, new databases through a symlinked parent and dangling database
  symlinks were characterized before and remain green after.
- A frozen resource protocol measures one-file and five-file/200-changed-line
  capture, open, save/commit, read and reopen journeys, plus metadata-heavy
  (480 files), large single (1 MiB), large batch (8 MiB) and held-reader cases.
- README and roadmap status now describe the durable-capture library that exists;
  the CLI remains a configuration inspector. `evidence/P03/demonstration-intake.md`
  holds the current nonvisual journey.
- The repository's complete `bash scripts/ci/check.sh` passes at the stabilized
  revision.

## Failure evidence

`failure-notes.md` explains the seam and its limits. Key observations:

```text
controlled commit failure: error:Capture store is unavailable: disk I/O error
postcommit over-limit result: saved:2:over=true:retained=2212328
postcommit measurement failure: error:Capture 2 was committed, but its retained
storage could not be measured: Capture store is unavailable: Cannot measure ...
```

`read_only_database_storage_fails_explicitly` covers a read-only database file
and directory returning an explicit `Unavailable`.
`process_killed_during_commit_is_complete_or_absent` stops a child at the real
WAL `fdatasync` inside commit, then kills it and reopens the store: the observed
outcome is `complete` with counts `(2,3,3)`, and both records read with exact
bytes and integrity. `fault-commit.txt`, `fault-during-commit.txt` and
`fault-postcommit.txt` retain the runs. The interposer is
`tests/support/records/io_fault.c`, built by the test with the existing C
compiler; it is not part of the library. It proves process-level error handling
at SQLite's write/fsync boundary, not device or power-loss behaviour.

## Path simplification evidence

Characterization ran before the change and is retained:

- `path-alias-characterization.txt` — file and directory aliases;
- `path-new-file-characterization.txt` — new database through a symlinked parent;
- `path-dangling-characterization.txt` — dangling database symlink.

After opening the caller path, `open_with_budget` canonicalizes the now-existing
database once and uses that identity for `-wal`/`-shm`/`-journal` accounting.
All 38 records tests remain green; the alias admission still measures canonical
files and the postcommit usage equals that footprint.

## Resource evidence

`resource-protocol.md` freezes inputs, host/load/cache conditions, run counts,
percentiles, raw values, stored bytes and peak/tree observations; raw output is
`measure-final.txt`. Summary from the final run:

| Journey | p50 | p95 |
| --- | ---: | ---: |
| one-file capture only (30) | 61,448 µs | 69,155 µs |
| five-file capture only (30) | 91,686 µs | 110,034 µs |
| one-file save+commit (20) | 409 µs | 545 µs |
| five-file save+commit (20) | 768 µs | 883 µs |
| one-file read (20) | 231 µs | 288 µs |
| reopen (20) | 276 µs | 358 µs |
| large-batch save+commit (1) | 145,470 µs | — |

The 8 MiB batch retained 16,970,168 bytes; the 480-file metadata case retained
181,096 bytes from 1,146 payload bytes. Storage and tree-RSS peaks are sampled
at 1 ms and are lower bounds; the tree includes Git children when sampled. No
future product budget is claimed.

## Checks

All commands ran from the worktree root with `PATH=$HOME/.cargo/bin:$PATH`.

| Command | Result |
| --- | --- |
| `bash scripts/ci/check.sh` with `CI_BASE_SHA=45dac3c...` | passed (`ci-check.txt`) |
| `cargo test --frozen --test records` | 38 passed (`records-all.txt`) |
| `cargo test --frozen --test records_demo` | 2 passed (`demonstration-output.txt`) |
| `cargo test --frozen --test records_measure` | passed (`measure-final.txt`) |
| `cargo clippy --frozen --all-targets -- -D warnings` | passed (`clippy-01.txt`) |
| `git diff --check` | clean |
| MDD state validation | valid, unchanged |

## Limits and unrun

- The commit-failure seam is a process-level WAL write error, not power loss,
  device failure or a lying `fsync`.
- Resource sampling misses sub-millisecond peaks and very short Git children;
  RSS includes the test harness; one host/load snapshot; warm OS caches.
- The admission reservation remains a conservative engineering allowance under
  bundled SQLite/runtime assumptions, not a hard quota or arbitrary-build proof.
- No resident/analyzer/report resources, hosted trial, publication or final
  independent review were performed.
