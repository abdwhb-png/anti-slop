# Current whole-process resource observations — P04

This document supersedes the incorrect whole-tree-memory claim in `../P03/resource-protocol.md` and in `../P03/implementation-report-r2.md`. It renews the affected measurements with the corrected observer from packet AS-M004-P04. Every P03 log, protocol and report remains unchanged as a historical observation of the pre-correction harness; only the claims identified under "Supersession" below lose their applicability.

## Protocol and inputs

- Repository `anti-slop`, branch `fm/as-m004-durable`, base and HEAD `45dac3c241338c94166369fe821fe1d3499cf974`; all AS-M004 changes uncommitted in this worktree, exactly as the final-r1 review recorded them. The only packet change to the measured source is `tests/records_measure.rs`.
- Measured source: `measurement-source-after.raw.rs.txt`, 24,827 bytes, SHA-256 `1aa6fdeb1e7779055f0141fbfd252f1ef5062bc47b8073052a7df147b298a6fd`. The pre-packet source is `measurement-source-before.raw.rs.txt`, SHA-256 `73ffa7ac07559ea87f2d91f70850fa6a9957e3235a5c5b80aa7976858dfa5379`; `measurement-source.diff.txt` is their unified diff.
- Command: `/home/abdwhb/.cargo/bin/cargo test --frozen --test records_measure -- --nocapture --test-threads=1`. Exit 0, 2 tests passed in 10.41 s. Raw output: `measure-current.txt`.
- Host and freshness: `Linux 7.0.0-42-generic x86_64 GNU/Linux`, 4 available CPUs, load average `0.63 0.92 1.13` at the run. Warm OS caches after fixture creation and repeated local runs; this is not cold-disk measurement. One host and one load snapshot only.
- Toolchain: `rustc 1.96.1 (31fca3adb 2026-06-26)`, `cargo 1.96.1 (356927216 2026-06-26)`; bundled SQLite with WAL and `synchronous = FULL` as the existing records implementation configures.
- Fixtures are the unchanged P03 fixtures: one modified tracked file (208 retained bytes); five modified files with 40 changed lines each (5,286 retained); 480 small untracked files (1,146 retained, 480 changes); one 1 MiB file (1,048,762 retained); eight 1 MiB files (8,388,794 retained); a held SQLite reader over two 1 MiB saves. Printed bounds remain 512 files, 1,048,576 bytes per file, 16,777,216 total bytes, 1,048,576 configuration bytes, default admission budget 10,737,418,240 bytes.
- Runs: 30 capture-only observations per one/five-file journey; 20 open/save/read observations per save journey; 20 reopen observations; one peak-monitor run per resource case; one combined capture+save journey for one file, five files and the large batch; one held-reader run. Percentiles are nearest-rank; all raw values are in `measure-current.txt`.
- The regression fixture that holds four blocked Git children shares a mutex with this protocol, so it is never resident during these workload measurements.

## Latency observations (microseconds)

| Journey | Runs | p50 | p95 |
| --- | ---: | ---: | ---: |
| one-file capture only | 30 | 61,675 | 75,362 |
| five-file capture only | 30 | 89,659 | 100,420 |
| one-file `Store::open` (fresh schema) | 20 | 716 | 916 |
| one-file save + synchronous commit | 20 | 393 | 466 |
| one-file read | 20 | 200 | 398 |
| one-file capture + save + read | 20 | 62,469 | 90,485 |
| five-file `Store::open` | 20 | 702 | 931 |
| five-file save + synchronous commit | 20 | 699 | 1,470 |
| five-file read | 20 | 359 | 439 |
| five-file capture + save + read | 20 | 92,720 | 148,241 |
| reopen existing store | 20 | 288 | 360 |
| large-batch capture + save | 1 | 614,575 | — |
| large-batch save + synchronous commit | 1 | 145,843 | — |

Capture dominates every full journey because it launches real Git children. `save + synchronous commit` includes the SQLite `FULL` commit path; a separate fsync-only duration was not isolated. These values are consistent with P03's latency observations and are one host/load sample; they establish no ADR-007 B1–B4 budget.

## Storage and memory observations

| Case | Payload bytes | Retained bytes | Save (µs) | Storage peak | Whole-owned-process tree RSS peak |
| --- | ---: | ---: | ---: | ---: | ---: |
| one file (save only) | 208 | 90,456 | 217 | 90,456 | 22,163,456 |
| five files (save only) | 5,286 | 102,816 | 445 | 102,816 | 22,163,456 |
| metadata-heavy, 480 (save only) | 1,146 | 181,096 | 12,946 | 181,096 | 22,167,552 |
| large single, 1 MiB (save only) | 1,048,762 | 1,149,296 | 18,647 | 1,149,296 | 23,502,848 |
| large batch, 8 MiB (save only) | 8,388,794 | 16,970,168 | 145,843 | 16,970,168 | 24,461,312 |
| one file, capture + save | 208 | 90,456 | 59,755 total | 90,456 | 34,279,424 |
| five files, capture + save | 5,286 | 102,816 | 87,728 total | 102,816 | 34,279,424 |
| large batch, capture + save | 8,388,794 | 16,970,168 | 614,575 total | 16,970,168 | 52,887,552 |
| held reader, two 1 MiB saves | 1,048,762 | 3,316,416 | 47,252 | 3,316,416 | — |

The process reported `VmHWM` 31,848 KiB and `VmRSS` 20,748 KiB at the end of the protocol. Storage peaks equal the final retained size in every case, which is the expected 1 ms-sampling lower bound for these fast writes; P02's longer checkpoint test remains the evidence that the sampler can observe an intermediate storage peak.

Capture+save journeys are strictly larger than save-only journeys because they contain the real capture's Git children. The corrected observer reported them explicitly: `git_children_peak=1`, `git_via_leader_peak=0`, `git_via_other_thread_peak=1` on all three combined journeys, with example Git children listed only under TID 2372415 of process 2372414 (one file: PID 2382930; five files: PID 2383024; large batch: PID 2383089). The save-only cases launch no Git child, so their values are unaffected by the defect and remain the process+harness figure.

## Supersession and applicability

- Superseded claim: `../P03/resource-protocol.md` states that the tree sampler "includes the test harness, the store process and any live Git children". That is false for every capture journey, because the sampler read only the leader thread's children list while libtest ran the test body on a worker thread. The same systematic omission underlies the "Tree RSS peaks sampled 21–32 MB" summary in `../P03/implementation-report-r2.md`.
- Superseded value: the P03 combined large-batch tree RSS of 32,600,064 bytes. The corrected current value for that journey is 52,887,552 bytes. The omitted ~20.3 MB is the capture's Git children, not a product-memory regression.
- Still applicable: all P03 behavioral, integrity, schema, fault-injection, admission/concurrency and demonstration evidence, which the final-r1 review examined and which this packet does not touch. Storage and latency figures are renewed above and agree with P03 within this host's noise. P03's save-only tree values are not invalidated by the observer defect because those phases spawn no child; they are nevertheless renewed here for one consistent current snapshot.
- Historical: P03 logs, protocols and reports stay byte-for-byte unchanged as observations of the pre-correction harness. They must not be cited as whole-process memory evidence for capture journeys.

## Limits

Sampling is 1 ms for both storage and memory, so short-lived children and sub-millisecond peaks can be missed; recorded peaks are lower bounds, and a missing sample is a sampling limit rather than evidence that no child existed. Summing per-process `VmRSS` counts memory shared between processes more than once, so the tree total is biased upward and is not a unique-page figure. A child that exits between being listed and being read contributes nothing; that death race is not treated as unavailable observation. Process-id reuse inside one sampling window is not observed but not excluded. OS caches are warm, the build is the debug profile, and there is one host, one load snapshot and small run counts. These observations do not establish cold-disk behavior, device-level durability, or any ADR-007 future analyzer/model/resident/report budget.
