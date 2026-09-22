# ADR-006: Output formats and shared projection

## Status

Accepted. Analysis reports and their encoders are not implemented.

## Date

2026-09-22

## Context

The user values AXI's compact, structured output for agent consumption while also needing ordinary integrations, CI artifacts and training data. Making a CLI encoding the internal schema or provider prompt would couple unrelated responsibilities and create competing interpretations of the same result.

## Decision

Use **TOON as the default CLI report format**, with **explicit JSON available in the first version**. Use **JSONL for telemetry/dataset interchange**, with complete evidence as defined by [ADR-005](005-local-evidence-store-and-dataset-exports.md).

The canonical, versioned data contract is owned by Rust. Construct one report projection, then encode it. At identical content-selection options, TOON and JSON must carry the same semantics, schema version, statuses, coverage and counts. Formatting cannot change findings or CI policy. Select output format explicitly rather than switching it according to terminal detection.

Apply the adopted AXI principles at the output boundary: bounded default views, explicit additional detail, truthful counts/truncation and machine-readable status/error information. Detail selection is separate from encoding. Operational diagnostics belong on stderr; structured report content belongs on stdout.

Keep observed findings, incomplete execution, freshness and recording state distinguishable. A report with zero displayed findings does not imply a complete clean analysis. Filtering or truncation must not hide the policy outcome or the scope that remains unexamined.

The model adapter chooses a suitable input representation independently of the CLI format. Dataset exports derive from retained records and evidence rather than the compact report projection.

## Alternatives Considered

- **TOON only:** compact output, but unnecessary integration friction when consumers already handle JSON.
- **JSON only:** straightforward interoperability, but does not preserve the user's selected default for agent consumption.
- **Separate business logic per encoder or model:** invites divergent findings, counts and policy decisions for the same run.
- **Treat displayed reports as the training dataset:** loses omitted evidence, rejected candidates and label provenance.

## Consequences

Qualify semantic equivalence through the real public boundary, including empty results, partial coverage, failures, Unicode, multiline/control characters, numeric values and truncated views. Encoder round trips through one library alone do not prove conformance or independent interoperability.

Choose and declare a TOON specification profile before claiming support. The bootstrap's `toon-format` candidate is not yet qualified; its installed presence or Serde compatibility would not settle specification differences. Measure encoding cost against [ADR-007](007-performance-budgets-and-qualification.md).

## Evidence and Remaining Qualification

Consolidates D6 and the [historical AXI analysis](../brainstorming-decisions/2026-09-22-anti-slop-design.md#cli-output-and-axi). The decision adopts selected output principles; it does not claim complete AXI conformance.

Exact CLI flags, view defaults/limits, schema versioning, error envelopes, codec/library version and compatibility policy remain to implement and qualify. The historical `--format` and `--full` examples describe intended ergonomics, not existing commands.
