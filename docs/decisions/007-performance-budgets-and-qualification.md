# ADR-007: Performance budgets and qualification

## Status

Accepted targets. No end-to-end implementation has been measured against them.

## Date

2026-09-22

## Context

The user chose Rust for speed and memory control, including repeated use by a future Pi package. Native extraction timings omit source capture, evidence building, recording and child processes. Selecting a fast language or moving work into a background process does not establish acceptable user latency or total cost.

## Decision

Treat the following as initial acceptance targets on declared representative workloads. Keep measurement boundaries, enabled rules and achieved coverage visible. Do not satisfy a target by silently dropping required work.

| Budget | Target | Measurement boundary |
| --- | --- | --- |
| B1 — Available report | p95 ≤ **50 ms** | Public resident request through receipt of the default bounded report, including store access, freshness/status projection and serialization. No analyzer or model invocation. Freeze report contents and record bytes/counts. |
| B2 — Small change, loaded analyzers | p95 ≤ **1 s** | Capture, invalidation, required local analysis, candidate/evidence work, durable recording and completed report. Replay controlled provider responses at the external seam to exclude remote waiting only; include their processing and recording. |
| B3 — First local analysis | p95 ≤ **2 s** for TypeScript or PHP; **10 s** for Rust | B2's local path plus application/native startup and first project load. State filesystem-cache conditions: process-cold with warm OS caches is not cold-disk measurement. |
| B4 — Session memory | **256 MiB** for TypeScript or PHP; **768 MiB** for Rust; **1 GiB** for mixed-language sessions | Whole owned process tree, including native workers, captured facts, buffers and database activity. Measure startup/active peak and retained idle use. Independent sessions add up; this is not a host-wide quota. |

Initial small-change cases are one source file with dependent consumers and a batch of up to five source files/200 changed lines. This bounds the edit scenario, not the evidence scope. Record actual changes and affected context. Configuration/dependency reloads and larger scans are separate scenarios; do not relabel an ordinary edit as cold because an adapter restarts unnecessarily.

Always report actual end-to-end time, remote wait/retries and recording cost alongside controlled local-path timings. B1 does not establish the future Pi hook's overhead. Core-only memory does not establish B4; disclose sampling, shared-page double counting and missed-peak limitations when using RSS measurements.

### Model phase

Use a configurable total deadline of **10 seconds interactive** and **60 seconds in CI** for the model-assessment phase of one analysis. Include admission/queueing, every candidate request, backoff and retries from the start of that phase. Parallel requests, fallback providers and retries do not receive fresh deadlines.

At expiry, stop scheduling, cancel supported outstanding requests, retain completed findings and mark missing assessments timed out/incomplete. Preserve ambiguous remote attempt and cost provenance; local cancellation does not guarantee remote execution or billing stopped. These are maximum waiting budgets, not p95 model-speed or accuracy targets.

### Reference workloads

| Workload | Repository | Role |
| --- | --- | --- |
| K1 | Pi-lens (`shared-services/pi-lens`) | TypeScript and repeated interactive use. |
| K2 | `quickcoin-platform` | PHP/TypeScript with declared framework and dependency scope. |
| K3 | Zerobox (`shared-services/sandboxes/zerobox`) | Rust multi-crate/dependency structure. |

Use these repositories alongside controlled fixtures. Their names identify workloads, not portable absolute paths; configure local locations outside committed fixtures. Freeze exact revision plus dirty/relevant untracked inputs, dependency/toolchain identity, file/byte counts, configuration, rules, expected coverage and edit scenarios. Use controlled captures without changing the user's active checkout.

The performance repositories are not an adjudicated rule-quality corpus. Accuracy and blocking qualification remain owned by [ADR-003](003-evidence-first-rules-and-selective-blocking.md).

## Alternatives Considered

- **Native query time or Rust-core RSS alone:** cheap measurements, but omit substantial user-visible work and owned resource cost.
- **Only cold benchmarks:** fail to test the repeated-use benefit that justifies client-owned residency.
- **Only warm foreground latency:** can conceal startup, retained memory, background work and cancellation costs.
- **Waive Rust budgets while rust-analyzer is provisional:** contradicts the reason for retaining a replaceable adapter rather than accepting unlimited overhead.

## Consequences

Qualification must cover startup, unchanged reads, small/repeated edits, reloads, new/deleted callers, cancellation/supersession and idle retention. Report raw observations, run counts, p50/p95, host/load/cache conditions, process ownership and completeness. Reuse native state only within bounded, measured resource use.

Targets are distinct from enforced limits. A limit hit produces explicit incomplete/unavailable work; a passing cropped benchmark or configured timeout is not proof of acceptance. If a target fails, improve the implementation/adapter or obtain an explicit decision to revise the target.

## Evidence and Remaining Qualification

Consolidates D8, D14, D15 and D17. Detailed experiment ideas remain in the [historical architecture proposal](../brainstorming-decisions/2026-09-22-anti-slop-architecture.md#performance-and-the-future-pi-consumer).

Freeze the benchmark protocol and environments before measurement. The earlier suggestion of 30 process-cold runs and 100 warm observations across five sessions is a starting protocol, not an established tail-confidence result. Whole-path and whole-tree measurements on the supported Linux/WSL2 environments remain undone.
