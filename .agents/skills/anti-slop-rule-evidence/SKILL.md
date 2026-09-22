---
name: anti-slop-rule-evidence
description: Design and validate Anti-Slop rule evidence with supporting facts, legitimate counterexamples, explicit uncertainty and scoped blocking eligibility. Use when adding, changing or qualifying R1-R9 candidate selection, evidence packets or assessment semantics; skip unrelated documentation, CLI metadata and dependency work.
---

# Anti-Slop Rule Evidence

## When to Use

Use this workflow to design, implement or assess a rule's candidate selection, evidence gathering, counterexamples or qualification. It applies to one bounded rule change or a deliberately chosen cross-language contract.

## When Not to Use

Skip it for a README typo, CLI help/version metadata, formatting, dependency installation or general release planning. A request to explain or review a rule does not authorize implementation, repository-wide analysis, model calls or blocking activation. Follow the requested task type.

## Establish the assessment

A structural pattern becomes useful only when tied to a concrete complexity problem. Define the question before selecting an algorithm so the candidate detector does not quietly redefine the rule.

1. Read the relevant rule in [ADR-003](../../../docs/decisions/003-evidence-first-rules-and-selective-blocking.md) and inspect the current `rules`, `analysis`, `judges` and `policy` owners as needed. Treat reserved directories as unimplemented, not available interfaces.
2. State the rule/version, language scope, actual maintenance burden being assessed and legitimate behavior that should survive.
3. Separate deterministic candidate signals from evidence sufficient to support the issue. For example, forwarding is a candidate for R6; authorization or dependency isolation may justify that same thin wrapper.
4. Keep the first-release scope unchanged when choosing a small implementation slice. Do not add new rule families or broaden R4 merely because another smell name is similar.

## Construct bounded evidence and counterevidence

False positives often result from leaving out why a boundary exists. Build a packet that permits both support and rejection, with uncertainty tied to exact missing facts.

1. Identify the declarations, callers, types, invariants, contracts or history needed for this question. Include relevant unchanged context; a diff alone may omit the responsibility that justifies the code.
2. Request actual adapter capabilities and declared scopes. Bind facts and spans to the analyzed generation, analyzer/configuration identity and source bytes.
3. Record unresolved dispatch, missing history, unsupported constructs, unmapped locations and acquisition limits. An empty reference result under partial resolution is not evidence of no callers.
4. Budget the packet and expose omitted context. If the decisive evidence is absent, keep the assessment insufficient/inconclusive instead of manufacturing a confident negative or positive.
5. Include counterevidence deliberately. One implementation is not proof of speculative abstraction; a legitimate lifecycle is not automatically a hidden protocol; fan-out is not proof of shotgun surgery.

R2, R7 and R9 can use some of the same facts, but each asks a distinct question. Reuse extraction while preserving rule meaning and grouping related actionable findings without inflating their count.

## Keep assessment, policy and labels separate

A provider verdict, a CI action and a training label answer different questions. Preserve the distinctions to make model replacement and independent evaluation possible.

1. Use the neutral outcomes from [ADR-004](../../../docs/decisions/004-provider-neutral-assessment.md): support, reject or insufficient evidence, separately from execution failure and policy action. Do not import provider response types into rule definitions.
2. Retain exact evidence and attributed execution records through the owner in [ADR-005](../../../docs/decisions/005-local-evidence-store-and-dataset-exports.md). Never use a truncated CLI view as the training source.
3. Keep model predictions, human/agent feedback, suppressions, deferred real issues and adjudication distinct. Suppressing a useful adapter warning does not by itself establish a verified negative label.
4. Assess blocking eligibility against the enabled rule's qualified language, project, analyzer/judge and evidence scope. Missing qualification or required evidence prevents a blocking claim; advisory results still disclose limitations.

## Validate through the real boundary

A positive fixture alone rewards overbroad detectors. Use discriminating examples and test the implemented public path rather than a copied helper.

1. For an authorized behavior change, run a minimal public-boundary test and observe the intended failure before implementing; use the repository's RED → GREEN → REFACTOR workflow.
2. Include a supported defect, a legitimate counterexample with a similar structural signal, and an ambiguous/partial-evidence case. Add only the language cases needed by the changed contract; record which other languages remain unqualified.
3. Use real production modules and controlled native fixtures. Mock provider transport or other external/nondeterministic seams, not the evidence and policy behavior being tested.
4. Check candidate recall separately from judge/policy precision. Rejected candidates, sampled non-candidates and manually found misses need provenance; analyzing only emitted findings cannot establish recall.
5. For blocking qualification, use independently adjudicated data and a frozen holdout; thresholds are selected on validation data. The performance repositories are not automatic truth labels.
6. For a design-only request, describe these cases and missing contracts without inventing executable success or creating production code.

## Report the bounded result

Return the rule question, evidence and counterevidence, coverage/unknowns, assessment versus blocking eligibility, checks actually run and remaining qualification. Save artifacts only when the task calls for them. A fixture or model agreement establishes its observed scope, not production readiness across R1–R9.
