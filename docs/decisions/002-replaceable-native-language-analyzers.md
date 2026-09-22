# ADR-002: Replaceable native language analyzers

## Status

Accepted. Native extraction and source-coherence experiments are bounded research evidence; production adapters remain unimplemented.

## Date

2026-09-22

## Context

Design rules need repository facts that syntax alone cannot establish: resolved relationships, types, callers, ownership boundaries and incomplete resolution. Rebuilding each language's semantics in Rust would add maintenance cost. Requiring one universal syntax layer would also obscure native capabilities and gaps.

## Decision

Allow the Rust core to use native libraries and external analyzers when their evidence quality and operational cost justify the boundary. Keep each integration replaceable behind `analysis`; consumers receive the observations and coverage needed by rules, not native ASTs or an invented universal type system.

Retain rust-analyzer provisionally for Rust, with its cost subject to [ADR-007](007-performance-budgets-and-qualification.md). The user's acceptance of its present trade-off does not waive performance targets or authorize unbounded resident instances. Defer Trellis O9; it is not an initial adapter.

The existing experiments supply the following implementation starting points. Versions identify measured inputs, not an automatically updated production support matrix:

| Language | Qualified starting point | Limits to preserve |
| --- | --- | --- |
| TypeScript | Oxc 0.150.0 for structure/local semantics; TypeScript 7.0.2 native LSP and a separate typed API path where necessary. | Oxc alone is not a complete project type graph. The structured API is unstable; hover text is not a substitute for structured type facts. Avoid duplicate native loads without a demonstrated capability need. |
| PHP | Mago 1.48.1 with its PHP SDK at 1.48.1 and the experiment's locked event-loop dependency. | Call-site span mapping and framework/dynamic resolution have gaps. Watch mode is not established as universally faster. |
| Rust | Official rust-analyzer release 2026-09-14, observed identity `0.3.3049-standalone`, through LSP. | The restricted experiment disables build scripts, procedural macros and other costly work. Macro expansion text does not establish exact original-source mappings; embedding richer HIR APIs is unqualified. |

A fact must carry enough provenance to identify the analyzer/configuration, actual source generation, query scope, unsupported or unresolved portions and source-mapping quality. Successful transport and an empty result do not establish complete analysis or absence of callers.

Bind facts to what the native analyzer actually consumed. Source bytes, native overlays, relevant configuration and dependencies must describe a coherent generation. Normalize spans against its exact bytes. A late result cannot become a current result merely because its request completed successfully. Unknown or stale consistency remains explicit and limits blocking eligibility.

Package tooling separately from analyzed projects. Ordinary analysis must use explicitly provisioned, verified analyzers; it must not install or update them. Preserve the selected dependency controls when provisioning a new integration. Temporary research installation permissions do not authorize a production installer.

## Alternatives Considered

- **Everything embedded in Rust:** potentially convenient distribution, but insufficient or unstable interfaces can make this a false constraint. Individual embedded adapters remain possible after qualification.
- **Tree-sitter or a universal AST required for every language:** useful syntax fallback in some cases, but not a replacement for native resolution and not mandatory here.
- **Trellis O9 now:** the bounded experiment's approximately 9.7-second median and 461 MiB observed process cost conflict with current priorities. These numbers describe that experiment, not every Trellis workload.
- **Replace rust-analyzer immediately:** the alternatives research did not establish a better qualified integration for the required facts. Keep the replacement boundary available and reassess with evidence.

## Consequences

Adapters require real fixtures for cross-file queries, legitimate boundaries, partial resolution and failure. Rules cannot interpret unsupported capability as a negative fact. Runtime and distribution requirements differ by language and remain part of total resource accounting.

Source consistency is a contract to prove, not an inference from an idle LSP server or unchanged disk hashes. The controlled overlay experiment demonstrates why those checks alone are insufficient.

## Evidence and Remaining Qualification

Consolidates D2, D7 and D9. Evidence: [language extraction](../research/2026-09-22-language-qualification/README.md), [source coherence](../research/2026-09-22-source-coherence/README.md), [rust-analyzer alternatives](../research/2026-09-22-rust-analyzer-alternatives.md) and [Trellis O9](../research/2026-09-22-trellis-o9/README.md).

Managed source views and the proposed SG1–SG6 protocol remain an implementation path to qualify against concurrent edits, overlays, dependency/configuration changes, late responses and real repository costs. The research does not establish whole-workspace atomic capture, hermetic dependency isolation or production performance. Exact bridge contracts, capability gaps and distribution remain open.
