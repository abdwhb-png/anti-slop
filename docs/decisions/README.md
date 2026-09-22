# Architecture decisions

This directory is the authoritative record of accepted architectural choices. It consolidates the decisions made during the initial design and approved scaffold preparation on 2026-09-22. The [root README](../../README.md#current-status) records what is implemented.

**Accepted is a design status, not an implementation or qualification result.** The Rust scaffold currently exposes help and version only. Each ADR states which details remain open. No PRD is maintained at this stage.

## Accepted records

| Record | Owns |
| --- | --- |
| [ADR-001: Rust engine and client-owned sessions](001-rust-engine-and-client-owned-sessions.md) | Package and module ownership; shared execution; client lifetime. |
| [ADR-002: Replaceable native language analyzers](002-replaceable-native-language-analyzers.md) | Native integration boundaries; capability and source-coherence requirements. |
| [ADR-003: Evidence-first rules and selective blocking](003-evidence-first-rules-and-selective-blocking.md) | R1–R9 semantics, counterexamples and blocking eligibility. |
| [ADR-004: Provider-neutral assessment](004-provider-neutral-assessment.md) | Judge replacement, outcome separation and model qualification. |
| [ADR-005: Local evidence store and dataset exports](005-local-evidence-store-and-dataset-exports.md) | SQLite authority, provenance, retention, quota and export integrity. |
| [ADR-006: Output formats and shared projection](006-output-formats-and-shared-projection.md) | TOON/JSON CLI equivalence, bounded views and JSONL interchange. |
| [ADR-007: Performance budgets and qualification](007-performance-budgets-and-qualification.md) | B1–B4, model deadlines and reference workloads. |

## Decision lineage

The existing D identifiers remain stable. They are historical decision references, not a second specification to maintain. Detailed accepted requirements now live in the linked ADRs.

| Earlier decision | Current owner |
| --- | --- |
| D1 — Selective CI blocking | ADR-003 |
| D2 — Rust core, external analyzers permitted | ADR-001, ADR-002 |
| D3 — Local collection and explicit aggregation | ADR-005 |
| D4 — Agent changes with repository context | ADR-001 |
| D5 — Initial rule and language scope | ADR-003 |
| D6 — TOON, explicit JSON, JSONL interchange | ADR-006 |
| D7 — Trellis O9 deferred | ADR-002 |
| D8 — Latency and memory constrain acceptance | ADR-007 |
| D9 — Provisionally retain rust-analyzer | ADR-002 |
| D10 — CLI and resident mode in the first version | ADR-001 |
| D11 — Resident lifetime follows the client | ADR-001 |
| D12 — SQLite authority | ADR-005 |
| D13 — Evidence retained until explicit purge | ADR-005 |
| D14 — Initial performance targets | ADR-007 |
| D15 — Whole model-phase deadlines | ADR-007 |
| D16 — Storage admission threshold | ADR-005 |
| D17 — Initial performance repositories | ADR-007 |
| D18 — Linux first, including WSL2 | ADR-001 |
| Approved bootstrap BP1 | ADR-001; installed state in the root README and Cargo files |
| Initial requirement: replace Jev and collect training evidence | ADR-004, ADR-005 |

## Remaining specifications

Resolve these when the corresponding work becomes concrete. An open item is not permission to invent a permanent contract silently.

| Item | Must be resolved before |
| --- | --- |
| Linux/WSL2 process-cleanup mechanisms and resource measurement | Claiming the selected hosts' resident lifecycle or resource guarantees. Platform scope is settled by D18. |
| CLI commands, change/base selection, protocol version, methods and compatibility rules | Publishing the first usable CLI/resident contract. Historical signatures and exit codes are proposals. |
| Source acquisition, dependency binding, cache invalidation and native capability gaps | Accepting a production adapter's evidence. The existing managed-view experiment establishes only bounded feasibility. |
| SQLite schema, journal/transaction strategy, export layout, quota accounting and purge mechanics | Claiming durable recording, self-contained exports or quota enforcement. |
| TOON codec and supported specification profile | Claiming interoperable TOON output. A candidate library is not a qualified codec. |
| Adjudicated rule corpus, label protocol and per-scope precision/recall/abstention gates | Enabling blocking or approving a replacement judge. Performance repositories do not provide labels. |
| Actual replacement model and its training interface | Building a vendor-specific training export. The “Laya” reference is not a resolved model identity. |
| Future Pi triggers and delivery cadence | Building that package. Repeated-use performance is already a core requirement. |

## Record changes

Create sequential `NNN-short-decision.md` files with Status, Date, Context, Decision, Alternatives Considered and Consequences. Use `Proposed` for unresolved choices and `Accepted` once the choice is authorized. Supporting evidence and remaining qualification should be explicit.

For a material change to an accepted choice, add a new ADR that identifies the superseded decision and its reason; update the old status and this index. Preserve old records. Clarifications and corrected links may be edited in place when they do not change the decision. Avoid duplicating mutable tables, thresholds or schemas across documents; link to their owning ADR or implemented contract.

The [historical design](../brainstorming-decisions/2026-09-22-anti-slop-design.md), [architecture proposal](../brainstorming-decisions/2026-09-22-anti-slop-architecture.md) and [bootstrap proposal](../brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md) retain the discussion and alternatives. Research files retain exact inputs, results and limitations. They do not override these ADRs or prove production readiness.
