# ADR-005: Local evidence store and dataset exports

## Status

Accepted. Storage, telemetry collection, feedback and export are not implemented.

## Date

2026-09-22

## Context

Normal analysis should produce the evidence needed to evaluate and train future judges. Logs, hashes and displayed findings alone cannot reconstruct an example or explain how its label was obtained. CI and local runs must contribute usable records without requiring a hosted collection service.

## Decision

Use local SQLite as the authoritative store for analyses, retained evidence and feedback. Derive self-contained JSONL/evidence exports for CI and explicit aggregation. `records` owns this behavior; technical tracing and CLI rendering must not become competing dataset writers.

Retain the information required to reconstruct and attribute each example:

| Record | Required meaning |
| --- | --- |
| Analysis | Exact captured source/evidence, snapshot and analyzer/configuration identity, facts and coverage, candidate/rule version, model identity, transmitted input and received output without credentials, latency/usage, execution, assessment and policy outcome. |
| Feedback | Exact target evidence/finding, actor kind, disposition, optional explanation, revision history and adjudication state. |
| Export | Versioned examples with retained evidence, transformation and label provenance, source identities and split membership. |

Keep actual evidence bytes; hashes identify content but cannot recover it. An export must contain, or package alongside itself, the evidence required to resolve its references without the originating database or workspace. Display truncation must not truncate dataset truth. Missing capture, persistence or export remains explicit even when analysis findings are available.

Preserve predictions, human feedback, agent feedback and adjudication as different sources. Suppression, a deferred real issue, a false positive, insufficient evidence, a rejected candidate, no candidate and execution failure cannot collapse into one negative label. A successful CI job is not ground truth.

Include rejected candidates, a controlled sample outside candidate selection with sampling provenance/probability, and manually reported misses. Otherwise candidate recall cannot be evaluated. Explicit aggregation preserves lineage and deduplicates related examples. Separate training and evaluation by related repository/code lineage and clone groups, with an appropriate time boundary; freeze the independent holdout before tuning.

### Retention and admission

Keep authoritative analyses, evidence and feedback until an explicit purge. Automatic cleanup is limited to reconstructible caches and technical logs. Export completion alone does not authorize deletion.

The default persistent-record budget is **10 GiB per workspace**, configurable. Warn at **80% (8 GiB at the default)**, then stop admitting new collection at the threshold. Do not silently delete evidence to continue collecting. Account for authoritative evidence plus SQLite auxiliary storage, including journal/WAL growth, and reserve space for admitted work. Reads and exports of retained data remain possible subject to actual storage availability.

Distinguish this admission policy from a hard filesystem quota. If storage or recording fails, report the resulting capture/durability state and apply required-check policy without changing a supported finding into a clean result. Transaction boundaries, concurrent reservations and threshold overshoot handling must be proven in the implementation.

## Alternatives Considered

- **Logs or CLI JSONL as the primary store:** convenient append paths, but poor ownership for feedback revisions, linked evidence, transactions and complete exports.
- **A hosted collector from the first release:** adds deployment and trust boundaries before local collection and explicit aggregation require them.
- **Automatic expiry or purge after export:** controls disk use but conflicts with retaining evidence until explicit deletion.
- **Provider-specific training records as the canonical schema:** ties collection to the current model and loses provenance needed for independent replacement evaluation.

## Consequences

Capture is a normal part of analysis and counts toward the performance budgets. Source content belongs in controlled local evidence and explicit exports, not routine terminal logs. Dataset exporters generate vendor-specific training views later without redefining stored facts or labels.

SQLite durability does not provide exactly-once external inference. Interrupted or ambiguous attempts retain their uncertainty and available cost provenance. Feedback acknowledgements and successful exports must correspond to actual durable results, not queued work.

## Evidence and Remaining Qualification

Consolidates D3, D12, D13 and D16 plus the original dataset requirement. The [historical records design](../brainstorming-decisions/2026-09-22-anti-slop-design.md#telemetry-feedback-and-datasets) explains the provenance distinctions.

Database schema, evidence blob placement, journal mode, migrations, export manifest/layout, atomicity, crash recovery, quota accounting and explicit-purge mechanics remain to specify and test. No SQLite dependency is installed in the initial scaffold. The quota and retention choices are accepted; their enforcement is not yet measured or implemented.
