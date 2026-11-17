# Frozen resource observations — P03

## Protocol and inputs

- Repository: `anti-slop`, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`; all AS-M004 changes were uncommitted in this worktree during measurement. The exact changed paths and dirty delta are listed in the P03 report.
- Command: `cargo test --frozen --test records_measure -- --nocapture --test-threads=1` (raw output `measure-final.txt`).
- Host: Linux, 4 available CPUs, load average `1.77 1.59 1.35` at the final run; warm OS caches after fixture creation and repeated runs. This is **not** cold-disk measurement.
- Toolchain: Rust 1.96.1; bundled SQLite 3.54.1; WAL with `synchronous = FULL`.
- Fixtures:
  - one file: one modified tracked file, 208 retained bytes.
  - five files: 5 modified files, 40 changed lines each (200 changed lines), 5,286 retained bytes.
  - metadata-heavy: 480 small untracked files (below the 512-file bound), 1,146 retained bytes, 480 changes.
  - large single: one 1,048,576-byte file, 1,048,762 retained bytes.
  - large batch: eight 1 MiB files, 8,388,794 retained bytes.
  - held reader: a real SQLite read snapshot over two 1 MiB saves, retaining WAL backlog.
- Runs: 30 capture-only observations per one/five-file journey; 20 open/save/read observations per save journey; 20 reopen observations; one peak-monitor run per resource case. Percentiles are nearest-rank; raw values are in `measure-final.txt`.

## Latency observations (microseconds)

| Journey | Runs | p50 | p95 |
| --- | ---: | ---: | ---: |
| one-file capture only | 30 | 61,448 | 69,155 |
| five-file capture only | 30 | 91,686 | 110,034 |
| one-file `Store::open` (fresh schema) | 20 | 773 | 1,034 |
| one-file save + synchronous commit | 20 | 409 | 545 |
| one-file read | 20 | 231 | 288 |
| one-file capture + save + read | 20 | 65,804 | 76,295 |
| five-file `Store::open` | 20 | 729 | 878 |
| five-file save + synchronous commit | 20 | 768 | 883 |
| five-file read | 20 | 405 | 450 |
| five-file capture + save + read | 20 | 97,756 | 110,277 |
| reopen existing store | 20 | 276 | 358 |
| large-batch save + synchronous commit | 1 | 145,470 | — |
| large-batch capture + save | 1 | 651,817 | — |

`save + synchronous commit` includes the SQLite `FULL` commit/fsync path; a
separate fsync-only duration was not isolated. Capture dominates the full
journey because it launches real Git children. The measurements are one
host/load sample and do not establish ADR-007 B1–B4 budgets.

## Storage and memory observations

| Case | Payload bytes | Retained bytes | Save (µs) | Storage peak sample | Tree RSS peak sample |
| --- | ---: | ---: | ---: | ---: | ---: |
| one file | 208 | 90,456 | 303 | 90,456 | 21,884,928 |
| five files | 5,286 | 102,816 | 464 | 102,816 | 21,884,928 |
| metadata-heavy (480) | 1,146 | 181,096 | 14,040 | 181,096 | 21,884,928 |
| large single (1 MiB) | 1,048,762 | 1,149,296 | 17,602 | 1,149,296 | 23,212,032 |
| large batch (8 MiB) | 8,388,794 | 16,970,168 | 145,470 | 16,970,168 | 24,174,592 |
| large batch capture + save | 8,388,794 | 16,970,168 | — | 16,970,168 | 32,600,064 |
| held reader, two 1 MiB saves | 1,048,762 | 3,316,416 | 36,541 | 3,316,416 | — |

At the end of the protocol the test process reported `VmHWM` 31,372 KiB and
`VmRSS` 20,476 KiB. The tree sampler includes the test harness, the store
process and any live Git children; it may miss very short child peaks. The
storage sampler also reads file metadata every 1 ms, so a transient peak between
samples is missed; recorded storage peaks are lower bounds (in several fast
cases the peak equals the final retained size). P02's 8 MiB checkpoint test
recorded an intermediate storage peak above its final value, confirming the
sampler can observe checkpoint duplication when the write lasts long enough.

The metadata-heavy case shows why source bytes alone are insufficient: 1,146
payload bytes produced 181,096 retained bytes because of SQLite rows, pages,
WAL and auxiliary files. The large-batch case documents database-plus-WAL
storage near twice the source payload.

The 512-file, 1 MiB/file, 16 MiB total and 1 MiB configuration bounds are the
existing `workspace`/`config` owners, printed in the raw output. The retained
source count is not the inventory file count: unchanged comparison candidates
are read but not retained, and a working/context path can share one observation.
The quota reservation remains an engineering admission allowance under the
declared bundled SQLite/runtime assumptions, not a hard quota or a formal proof
for arbitrary builds.

## Exclusions

No cold-disk, resident, analyzer, model, report or whole-mission budget is
claimed. No CI run is substituted for these measurements.
