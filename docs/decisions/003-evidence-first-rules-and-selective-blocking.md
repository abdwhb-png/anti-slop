# ADR-003: Evidence-first rules and selective blocking

## Status

Accepted. No rule or blocking policy is implemented or qualified for production.

## Date

2026-09-22

## Context

The tool should detect unnecessary design complexity without punishing useful interfaces, abstraction or comments. A smell name or structural count is not sufficient evidence of a defect. Unqualified blocking would impose false-positive costs on every change and undermine trust in the analyzer.

## Decision

The first usable release covers R1–R9 in TypeScript, PHP and Rust. Rules own their meaning and evidence requirements; language adapters supply facts and judges assess bounded questions. Use APOSD's concern with complexity, dependencies and obscurity to assess relevance, without inventing a universal complexity score.

| Rule | Assessment | Required counterexample or limit |
| --- | --- | --- |
| R1 — Narrating comments | Does the comment only repeat associated code? | Preserve rationale, constraints, invariants and non-obvious behavior. |
| R2 — Duplicated knowledge | Does the same decision or invariant have multiple maintenance owners? | Similar code for independent responsibilities is insufficient. |
| R3 — Primitive obsession | Are domain invariants lost, validations repeated or values easily confused? | Transport primitives and ordinary calculations can be appropriate; a value object is not automatically the remedy. |
| R4 — Speculative abstraction | Does machinery for hypothetical variation add current complexity without a useful boundary? | One implementation, a factory, generic API or hook alone is insufficient. A replaceable judge is a current requirement here. |
| R5 — Shallow modules | Does interface burden exceed the useful complexity hidden from callers? | Line count and method count do not answer this question. |
| R6 — Pass-through layers | Does delegation repeat essentially the same responsibility across layers? | Security, ownership and dependency isolation can justify thin adapters. |
| R7 — Information leakage | Must callers or independent modules know internal implementation decisions? | A necessary public contract is not automatically leakage. |
| R8 — Hidden protocols | Must callers coordinate implicit ordering or conjoined operations to preserve an invariant? | A useful, explicit lifecycle is not automatically a defect. |
| R9 — Shotgun surgery | Does one conceptual change require coordinated edits across independent locations? | Fan-out, co-change or one commit alone is insufficient; missing history limits the conclusion. |

Select candidates deterministically where possible, then gather bounded supporting and opposing evidence, including relevant unchanged code. Expose missing context instead of forcing a verdict. R2, R7 and R9 may share facts while asking different questions; related findings must not inflate the count of actionable issues artificially.

Only enabled rules validated on a corpus may block CI, and only within the validated rule, language, project characteristics, evidence coverage and analyzer/judge configuration. Other results remain informative. Qualification is not inherited merely because an API remains compatible or another language passed.

Keep rule assessment, execution completion and policy action distinct. Required checks that cannot complete surface failure/incompleteness according to the CI contract. Advisory incompleteness stays visible even when it does not fail the job. Zero reported findings is not a claim that every rule fully analyzed the repository.

## Alternatives Considered

- **Counts or pattern matches as automatic verdicts:** inexpensive candidate signals, but they confuse legitimate boundaries with design defects.
- **Block every model-supported finding:** ignores calibration, capability gaps and false-positive tolerance in the actual project scope.
- **Only comments or a smaller rule set for the first release:** a useful early implementation exercise, but not the selected usable-release scope.
- **Declared architecture contracts as the primary detector:** useful evidence when present, but insufficient for discovering emerging problems users did not anticipate.

## Consequences

Every rule needs positive, legitimate-negative and ambiguous examples across its supported languages. Measure candidate recall separately from judgment/policy precision; otherwise a precise judge can hide a candidate generator that misses most issues.

No numeric quality threshold has been accepted yet. Define the adjudicated corpus, label protocol and per-scope precision/recall/abstention criteria before enabling blocking. Choose thresholds on validation data and evaluate on a frozen independent holdout. [ADR-005](005-local-evidence-store-and-dataset-exports.md) owns provenance and dataset separation.

## Evidence and Remaining Qualification

Consolidates D1 and D5 and the clarified rule meanings in the [historical design](../brainstorming-decisions/2026-09-22-anti-slop-design.md#rule-semantics). The four explicitly requested smells are represented by R2, R3, R4 and R9.

Exact candidate algorithms, rule versions, finding grouping, suppression behavior and CLI exit values require implementation contracts and tests. No historical example score is an accepted blocking threshold.
