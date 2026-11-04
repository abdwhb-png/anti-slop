# Anti-Slop Product Requirements Document

Date: 2026-09-22

Status: **Product workflows and acceptance criteria approved.** The user confirmed D19–D22 and approved the workflows in Section 2 and qualification requirements in Section 3 on 2026-09-22. Subsequent configuration choices D23–D25 are recorded in ADR-008. Detailed implementation contracts and evaluation-profile settings remain open in Section 5; the post-MVP roadmap is indicative. Approval of these requirements is not implementation or successful production qualification.

This document owns product workflows and release acceptance. The [accepted ADRs](decisions/README.md) own architectural choices, rule meanings and existing resource policies. The [root README](../README.md#current-status) records implemented behavior. Referenced requirements are incorporated without copying their mutable tables; an architectural change still needs an explicit decision record.

## 1. Executive Summary

### Problem Statement

Agent-produced changes can introduce duplicated knowledge, unnecessary abstraction and other design complexity that ordinary correctness checks do not establish. Developers need reviewable evidence with a controlled false-positive burden, while preserving the original inputs and feedback needed to improve the semantic judge over time.

### Proposed Solution

Build a Rust tool that analyzes explicitly selected changes with repository context, combines deterministic evidence extraction with bounded semantic assessment, and exposes the same engine through CLI, CI and client-owned resident sessions. Collect local evidence and attributed feedback during normal use so future models can be evaluated and substituted without redefining the product's rules or dataset.

### Scope and Release Boundary

The first usable release covers **R1–R9 in TypeScript, PHP and Rust**, with rule meanings and legitimate counterexamples defined in [ADR-003](decisions/003-evidence-first-rules-and-selective-blocking.md). It targets Linux, including WSL2, and provides both invocation modes selected in [ADR-001](decisions/001-rust-engine-and-client-owned-sessions.md). Product surfaces and project documentation are in English.

**D20 — Confirmed:** at least one rule/language combination must be qualified and available for selective CI blocking in the first usable release. Other combinations may remain advisory within their declared capabilities. The entire R1–R9/language scope still needs implemented evidence and assessment paths; an unsupported family or language cannot be counted as delivered merely because it returns an unavailable status.

An internal vertical slice may cover fewer cases while the product is being built. It is not the first usable release. The present executable exposes help/version only.

### Success Criteria

These are acceptance targets, not achieved results. Report the workload, scope, versions and denominators alongside every measurement.

| KPI | Target | Measurement and release evidence |
| --- | --- | --- |
| KPI-01 — Blocking precision (D21, confirmed) | **At least 95% precision** among blocking findings in each qualified scope. At least one rule/language combination qualifies for the first release. | Independently adjudicated true positives divided by adjudicated true positives plus false positives, after grouping duplicate reports of the same actionable issue. Evaluate on a frozen holdout for the applicable rule/language/project/analyzer/judge profile. A model's confidence is not this metric. |
| KPI-02 — Latency | Meet **B1–B3** under the exact boundaries in [ADR-007](decisions/007-performance-budgets-and-qualification.md#decision). | Public report reads, warm small-change analysis and first local analysis on the frozen reference workloads. Include recording and serialization; separately report real provider wait and end-to-end time. |
| KPI-03 — Memory | Meet **B4** in [ADR-007](decisions/007-performance-budgets-and-qualification.md#decision). | Startup/active peak and retained idle memory for the entire owned process tree, with the accounting method and sampling limits disclosed. |
| KPI-04 — Reconstructible records | **100%** of examples in an export reported complete resolve their required evidence and provenance without the originating workspace or database. | Export/import checks in an isolated location, including source bytes, provider exchanges, label origin and feedback revisions. Missing content makes the export incomplete or failed. |
| KPI-05 — Truthful outcomes | **Zero** false complete/clean outcomes or unqualified blocking actions in the required failure and policy acceptance cases. | Exercise missing scope, partial facts, stale results, unavailable judges, timeouts, cancellation, quota exhaustion and failed persistence/export through the public boundaries. |

KPI-01 is a target on evaluated findings, not a guarantee that exactly one in twenty future findings will be false. It does not measure missed defects. Candidate recall, end-to-end recall, abstention and statistical uncertainty must also be reported and assessed before a profile qualifies. Their acceptance thresholds and required sample sizes remain open; a tiny perfect sample or a single emitted finding cannot qualify a profile.

## 2. User Experience & Functionality

### User Personas

| Persona | Need |
| --- | --- |
| Developer working with coding agents | Review the selected changes, inspect why a finding matters, and distinguish useful design boundaries from unnecessary complexity. |
| Repository maintainer configuring CI | Enable only qualified blocking behavior and receive distinct evidence of violations, incomplete checks and operational failures. |
| Agent client or integration author | Request bounded analyses, reuse a client-owned session and consume current reports without triggering hidden work. |
| Maintainer curating data and evaluating judges | Recover exact examples, attribute feedback, preserve independent evaluation data and compare a replacement judge. |

The developer and maintainer may be the same person. These personas do not imply a hosted account system or a multi-user application.

### User Flow

Provision verified tooling separately → select a workspace and an explicit analysis scope → resolve inputs and disclose capabilities → capture native evidence → assess eligible candidates → apply qualified policy → publish a bounded report and recording status → inspect/record feedback → explicitly export or evaluate retained examples.

A resident client reuses the same analysis operations. Reading an existing report is a separate operation and never starts analysis or model inference.

### User Stories and Acceptance Criteria

#### US-01 — Select exactly what is analyzed

As a developer, I want to select the changes to analyze so that Anti-Slop evaluates the work I intend to review.

**D19 — Confirmed:** every analysis request requires an explicit scope; there is no default analysis scope.

**Initial CLI contract (L01.1):** `anti-slop analyze` requires exactly one of these selectors:

| Selector | Selected change bytes |
| --- | --- |
| `--staged` | Compare HEAD to the index and select the index snapshot. Working-tree edits beyond the index and untracked files are not selected changes. |
| `--unstaged` | Compare the index to the current working tree, including the current bytes of nonignored untracked files. Staged-only changes are not selected changes. |
| `--local` | Compare HEAD to the current working tree, including the current bytes of nonignored untracked files. |
| `--base <ref>` | Compare the supplied base directly to the current working tree, including local changes and the current bytes of nonignored untracked files. |

- Resolve and disclose the supplied `--base` once. Do not substitute a merge-base, branch default or implicit selection.
- `anti-slop report <run-id>` reads retained state only; it requires no analysis selector and performs no source capture, native analysis or model call. Preserve the existing help and version operations.
- Reject a missing, unknown, invalid or conflicting analysis selector before analyzer or model work. Do not fall back to the current branch, a previous analysis or a configured default silently.
- Report selected change bytes separately from supporting repository context for every selector. Context may inform extraction or assessment but does not broaden the selected changes; attribute it to its own source generation. For example, `--staged` uses index bytes even when the working-tree copy differs, and any working-tree bytes used as context remain identified as context.
- Resolve and report the effective selection, supplied base identity where applicable, and captured source identity. Preserve the working tree. Detect unstable acquisition and state the achieved consistency/freshness scope; a commit hash alone is not the complete local input identity.
- Help and report-reading operations require no analysis scope because they do not analyze code.

#### US-02 — Understand findings and uncertainty

As a developer, I want evidence and counterevidence for each finding so that I can decide whether the code should change.

- Identify the rule, captured source location where accurately mapped, actual observations, attributed assessment and relevant counterevidence. Missing or approximate mappings remain explicit.
- Provide bounded report views with truthful counts, truncation and access to further detail. TOON and explicit JSON must express the same selected report under [ADR-006](decisions/006-output-formats-and-shared-projection.md).
- For analysis and retained-report reads, follow ADR-006's TOON default and explicit JSON choice. Encoding must not change findings, scope or policy; codec profile and detailed view limits remain under OPEN-06.
- Keep execution, coverage, assessment, freshness, policy and recording independently observable. They are separate facts that may coexist, not one combined status. No candidates, a rejected candidate, insufficient evidence and unavailable execution are different outcomes.
- A request identifies its selected-source generation separately from supporting-context generations, its requested scope and the effective configuration used. A run is one attempt for that captured request; it does not imply that every step completed or that anything was retained. A later request after relevant source or configuration changes must not be represented as using the earlier inputs.
- A commit or byte hash can identify a revision or bytes but cannot recover their contents. A commit alone does not identify index, working-tree or untracked bytes. Retain or package the actual captured evidence required by [ADR-005](decisions/005-local-evidence-store-and-dataset-exports.md), and keep selected bytes distinct from supporting context. Unknown or unavailable source components remain explicit.
- Effective provenance identifies resolved non-secret settings and the relevant analyzer, dependency, judge, model and renderer versions actually used. Do not claim an unknown identity is known, and never retain credential values.
- Native facts, evidence and spans bind to the exact source generation and native overlay consumed, plus relevant configuration/dependency state, analyzer identity and query scope. A path and line alone do not establish that binding; partial resolution and approximate or unknown mappings remain visible.
- A finding/evidence occurrence identifies the particular observed evidence and candidate under its source/configuration generation and rule version. It is not identified by a rule name or location alone. Materially changed evidence is a distinct occurrence and does not silently inherit adjudication, as required by US-06.
- An attempt may have an identifier available for tracing before it is retained; that identifier does not claim a durable run record exists. Report attempt, request, source and configuration identities only when actually known. A durable run identity, durable-success acknowledgement or complete-record claim follows successful retention; failed admission or recording remains explicit.
- Preserve these separate obligations in reports and retained records without defining a universal status enum:
  - Execution says which work completed, failed, timed out, was cancelled or did not run.
  - Freshness says whether results still match the selected source and context generations; stale or superseded results remain attributable but are not current.
  - Coverage says which selected/context scope and analyzer capabilities were actually covered, including missing, unsupported and unresolved portions.
  - Assessment records the provider-neutral result for each candidate: supports the issue, rejects it, or finds evidence insufficient. A timeout or an assessment that was not requested is not an insufficient-evidence result.
  - Policy shows whether an enabled, qualified rule may act on current evidence. Stale evidence cannot authorize blocking.
  - Recording says whether evidence and outcomes were durably retained; it is distinct from execution and policy.
- Group related findings about one actionable issue without erasing their rule identities or inflating issue counts. Display filters do not change CI policy.
- A useful thin adapter or necessary comment must remain a legitimate counterexample. Provider confidence or explanatory prose is not additional extracted evidence.

#### US-03 — Enforce qualified CI policy

As a repository maintainer, I want selected rules to block only within their validated scope so that CI provides a trustworthy design gate.

- Apply the US-01 explicit selector contract in CI; CI environment metadata must not silently choose another scope.
- Allow blocking only for an enabled rule whose qualification profile matches the rule/candidate selector, language/project characteristics, analyzer/evidence contract, effective judge/renderer and required coverage.
- Make at least one qualified combination available in the first release, subject to KPI-01 and the qualification procedure in Section 3. Shipping that profile does not automatically enable it in every repository.
- Preserve separate public outcomes for qualified violations and inability to complete required work. Required analyzer/judge/recording failures must not report a complete clean check; advisory incompleteness remains visible even when it does not fail the job.
- For the initial `anti-slop analyze` operation, use these process exit values:

  | Exit | Analysis outcome |
  | --- | --- |
  | `0` | No blocking violation and no required operational failure. Advisory incompleteness remains visible in the report. |
  | `1` | At least one qualified blocking violation and no required operational failure. |
  | `2` | Invalid analysis input, including a missing, conflicting, invalid or unresolved selector; reject it before analyzer or model work. |
  | `3` | Required operational failure. If a qualified blocking violation also exists, retain both outcomes in the report and return `3`. |

- These analysis exits do not define exit behavior for the read-only report command or future resident methods.
- Produce self-contained CI artifacts when export is requested. An artifact or recording failure cannot be reported as a complete dataset capture.

#### Initial request and outcome examples (L01.1)

- **V01 — Local and base selections:** Let HEAD be `H`; let the supplied `release` ref resolve to `B`; and let `merge-base(H, B)` be `M`, with `H`, `B` and `M` all distinct. For `src/example.ts`, assume the content is `common` at `M`, `base` at `B`, `head` at `H`, `staged` in the index and `disk` in the working tree. Let `staged-only.ts` change from `old` at `H` to `staged-only` in both the index and working tree, and let `new.ts` be nonignored and untracked. `--staged` selects index bytes against `H`, including `staged-only.ts` and excluding working-tree-only edits and `new.ts`; `--unstaged` selects the index-to-disk change for `src/example.ts`, excludes the staged-only file whose index and disk bytes match, and includes current `new.ts` bytes; `--local` selects current disk bytes against `H`, including `staged-only.ts` and `new.ts`; and `--base release` selects current disk bytes directly against `B`, including `new.ts`, not against `M`. Each result identifies selected bytes separately from supporting context, including any working-tree bytes used as context for the staged selection.
- **V02 — Invalid request:** `anti-slop analyze` with no selector, `anti-slop analyze --staged --local`, `anti-slop analyze --unknown-selector`, `anti-slop analyze --base` or `anti-slop analyze --base missing-ref` (when that ref does not resolve) returns `2` before native or model work. Help, version and `anti-slop report <run-id>` do not require an analysis selector.
- **V03 — Legitimate design:** A comment that explains a non-obvious invariant and a thin adapter that isolates provider-specific response types can be justified design choices. Neither is a violation solely because it adds a comment or a small layer; the finding still requires the evidence and counterevidence defined by [ADR-003](decisions/003-evidence-first-rules-and-selective-blocking.md).
- **V04 — Required and advisory incompleteness:** A timeout, unsupported capability or stale source result stays visible as incomplete execution or freshness evidence, not as “no findings.” Required incompleteness returns `3`; advisory incompleteness remains visible and may return `0` when no blocking violation or required failure exists.
- **V05 — Qualification and coexistence:** An unqualified candidate cannot block. A qualified blocking violation returns `1` when required work succeeds. If that violation coexists with a required operational failure, preserve both in the report and return `3`.
- **V06 — Recording and empty selection:** A recording failure cannot receive a durable-success acknowledgement; when recording is required, it is a required operational failure and returns `3`. A valid empty selected set is visible and may return `0` without claiming native analysis ran.

#### Provenance and state examples (L01.2)

- **P2-E01 — No candidate, rejected candidate and legitimate negative:** A completed analysis with the required declared coverage and no emitted candidates records no candidates within that coverage; it does not claim universal absence. A candidate whose assessment rejects the issue remains a rejected candidate with that assessment. A legitimate negative is a separate attributed adjudicated example, not an inference from no candidate, rejection, insufficient evidence or execution failure.
- **P2-E02 — Required capability, native failure and stale evidence:** Suppose a current run has already retained a qualified finding from source generation S and then a required native query fails or reports an unsupported capability. Preserve the finding and its evidence alongside the required failure and uncovered scope; apply the existing required-failure outcome in US-03. If a late result instead refers to older generation S0, retain it as stale evidence and do not let it support a current blocking decision. Report source/configuration identities in either failure only when actually observed.
- **P2-E03 — Insufficient evidence and model timeout:** A completed assessment that cannot decide is “evidence insufficient” under [ADR-004](decisions/004-provider-neutral-assessment.md). A provider timeout is an execution failure with no completed support/reject/insufficient assessment. When model calls are disabled or a required judgment is unavailable, that missing assessment remains visible and cannot become a clean result.
- **P2-E04 — In-flight inputs and failed recording:** Once a request begins, later edits to source, configuration or dependencies do not mutate the inputs of its in-flight native/model work; a later request has its own effective identities, and an old result is not current for it. If retention fails, expose the recording failure and only identities actually known; any available attempt identifier remains useful for tracing but does not claim a durable run identity or acknowledgement. Provider credentials remain outside retained request/transport evidence.

#### US-04 — Reuse a session without stale results or leaked processes

As an agent client, I want a resident session so that repeated analyses reuse useful state within the resource budgets.

- Use the same rule, assessment, policy and records behavior as a one-shot CLI run. Every analysis still supplies an explicit scope.
- Bind the session to its client lifetime; verify graceful client close and abrupt parent termination on supported Linux/WSL2 environments, including owned descendants.
- Expose progress, cancellation and bounded existing-report reads. Distinguish cancellation requested from completed cleanup and preserve interrupted work when possible.
- A late or superseded result must not overwrite a newer current result. Reports identify their source generation and freshness.
- Bound queued requests, output and retained native state. A stalled reader or repeated edits cannot justify unbounded growth.
- Qualify report latency and full session resource cost against ADR-007. No cross-client broker or global daemon is required.

#### US-05 — Collect training evidence during normal work

As a maintainer, I want analyses to retain their evidence automatically so that useful examples do not depend on manually reconstructing past code.

- Use the authoritative store and provenance contract in [ADR-005](decisions/005-local-evidence-store-and-dataset-exports.md); technical logs and report previews are not alternative dataset writers.
- Retain actual analysis-relevant source/evidence, candidate and coverage information, versions, provider attempts/responses, usage/latency and policy outcomes. Exclude credentials from captured transport records.
- Preserve rejected candidates, insufficient evidence, no-candidate runs, controlled non-candidate samples and manually reported misses, with their sampling/selection provenance.
- Include collection and persistence in performance accounting. Failed capture remains visible and cannot be described as a complete training example.
- Retain authoritative records until explicit purge, following ADR-005's quota/admission policy. Reconstructible caches have a separate lifecycle.

#### US-06 — Give feedback without corrupting labels

As a developer or reviewing agent, I want to record the disposition of a finding so that later training can distinguish incorrect warnings from accepted trade-offs.

- Keep model predictions and model-to-model agreement distinct from human feedback, agent feedback and independently adjudicated labels. A Jev verdict, agreement with Jev, a successful CI run or absence of candidates never supplies an independent label.
- Attach feedback to an exact finding/evidence occurrence and record actor kind, disposition, optional rationale and adjudication state. Preserve false-positive reports, confirmed issues, suppression, accepted trade-offs, deferred real issues, insufficient evidence and execution or recording failures distinctly. None automatically becomes a positive or negative adjudicated label.
- Label provenance identifies the exact evidence occurrence and source generation, actor kind and adjudicator role, label-protocol revision and recorded decision. Keep disagreements and their provenance visible; correct feedback and adjudications through attributed revisions that preserve prior values. Do not resolve disagreement by overwriting a record or treating unapproved consensus as a label. Adjudicator selection and the disagreement-resolution process remain gated under OPEN-02.
- A durable acknowledgement follows persistence, not merely queuing.
- Support reporting a missed issue, with its source context, so evaluation is not limited to candidates the system already selected.
- Materially changed evidence does not silently inherit an adjudicated label from an earlier occurrence. A reviewed carry-forward mechanism needs explicit provenance.

#### US-07 — Export, aggregate and evaluate independently

As a dataset curator, I want complete portable exports so that local and CI examples can be aggregated and evaluated without their original machines.

- Export versioned JSONL with the evidence required to resolve every included example. Verify completeness after copying the artifact away from its originating workspace and database.
- Preserve provenance, distinct dispositions, feedback revisions, transformation identity and split membership. Do not turn an incomplete example into a negative label.
- Aggregate explicitly and preserve repository/code lineage across related revisions, clones and examples while deduplicating. A deduplicated view must retain each source example's identity, provenance and history, and all related lineage must remain within one immutable partition.
- Freeze an independently adjudicated, profile-specific holdout before tuning. Performance repositories are not automatically a labeled quality corpus; any use for quality evaluation requires separately declared selection, independent labels and lineage-aware partitions.
- Failed export or missing evidence is explicit. A successful export does not authorize deleting the local source records or uploading them elsewhere.

#### US-08 — Replace the judge without redesigning the product

As a tool maintainer, I want to introduce another judge so that model quality can improve independently of the rule engine and dataset format.

- Use the provider-neutral contract in [ADR-004](decisions/004-provider-neutral-assessment.md). A provider adapter does not change rule meaning, canonical evidence, feedback labels or CLI encoding.
- Record effective provider/model/renderer/configuration identity and raw provider observations. Confidence thresholds and qualifications do not transfer automatically.
- Verify the neutral boundary with a controlled alternative adapter; this does not require a second paid production provider in the first release.
- Before enabling a real replacement for blocking, pass contract checks, independent held-out evaluation, calibration and non-blocking comparison; retain a known qualified configuration for rollback.
- The future model called “Laya” remains unidentified. Selecting or training it is not required to deliver the replacement boundary.

#### US-09 — Control inference and storage side effects

As a repository maintainer, I want explicit inference and storage controls so that an analysis respects the intended data and resource boundaries.

- Apply D22 in Section 3. Provide an explicit mode without model calls; missing semantic assessments remain unavailable rather than becoming clean verdicts.
- Expose the effective provider, collection/transmission scope, resource limits and recording state through the relevant configuration/report surfaces. Credential values must never appear there.
- Before sending a new assessment request, ensure the required evidence/request can be retained. Quota exhaustion or failed admission stops new collection-dependent work without silently discarding evidence.
- Allow inspection and explicit export of existing records while collection is stopped, subject to actual storage availability. Raising a configured threshold, moving data and explicit purge are user actions.
- Respect the retention and storage thresholds owned by ADR-005 and the total model-phase deadlines owned by ADR-007. Retries and fallbacks cannot reset those deadlines.

### Non-Goals

- **NG-01:** Automatic code rewriting or replacing linters, type checkers, tests and security tools.
- **NG-02:** Additional languages, macOS or native Windows support in the first release.
- **NG-03:** The Pi package itself, automatic installation of agent hooks, a cross-client broker or a global daemon. Future Pi use informs present performance requirements.
- **NG-04:** A hosted telemetry service, web dashboard, general agent orchestrator or distributed job platform.
- **NG-05:** Training a production replacement model or guessing a vendor-specific training format before choosing that model.
- **NG-06:** Initial Trellis O9 integration or a mandatory universal AST/type system.

## 3. AI System Requirements

### Tool Requirements

Use the replaceable native boundaries in [ADR-002](decisions/002-replaceable-native-language-analyzers.md): Oxc/native TypeScript, Mago and provisionally rust-analyzer are the qualified research starting points. Preserve their recorded versions and capability limits when implementing adapters; research results are not production support claims.

Jev is the initial intended semantic provider. Rust owns candidate selection, bounded evidence assembly, outcome meaning, deterministic policy and durable records. The judge receives a specific question and evidence packet; it does not control extraction, execute repository instructions or decide CI policy.

### Inference Policy

**D22 — Confirmed:** after a remote provider is configured and source transmission is authorized for the repository, an analysis automatically requests judgments for candidates that need them, within configured limits. An explicit mode without model calls is available. This policy does not authorize model calls during PRD preparation.

Source and comments are untrusted evidence, not instructions. Validate the response structure, selected evidence references and provider identity. A timeout, malformed response, unknown capability or ambiguous remote attempt remains an execution/provenance state; none means that the code is clean.

Use the complete per-analysis model-phase deadline in [ADR-007](decisions/007-performance-budgets-and-qualification.md#model-phase), including queueing and retries. Retain usage and possible duplicate-cost provenance; local cancellation cannot guarantee that a remote provider stopped work or billing.

### Evaluation Strategy

1. Before constructing a corpus or tuning selectors, prompts, thresholds or models, identify the target population and representative project characteristics, intended evaluation scope, label/adjudication protocol and revision, immutable partition plan, and provenance for source selection and sampling. Record selection probabilities for sampled non-candidates. These declarations do not select the first qualifying rule/language profile or supply acceptance thresholds.
2. Build positive, legitimate-negative and ambiguous examples for every R1–R9/language combination in the applicable release scope. Exercise actual native facts and the public engine path; use controlled provider responses only when testing the surrounding contract. Keep model predictions, human feedback, agent feedback, suppression, deferral, adjudication, insufficient evidence and execution or recording failure separately attributable as required by ADR-004 and ADR-005.
3. Preserve repository, revision, clone and related-example lineage across aggregation. Keep every related lineage group wholly within one immutable training, validation or evaluation partition, while retaining each deduplicated example's identity, provenance and history. Freeze an independently adjudicated, profile-specific holdout before tuning candidate selectors, prompts, thresholds or models. Tuning on holdout results does not establish independent qualification.
4. Report candidate-generator recall as unique independently adjudicated relevant issues for which the candidate generator emitted a candidate divided by all unique independently adjudicated relevant issues in the declared evaluation population. Candidate-only examples cannot establish candidate recall.
5. Report assessment precision as independently adjudicated true issues among supported issue groups divided by all adjudicated supported issue groups, after grouping duplicate reports of the same actionable issue. Report end-to-end recall as unique independently adjudicated relevant issues surfaced as supported findings by the complete selection-and-assessment path divided by all unique independently adjudicated relevant issues in scope. For both recall metrics, distinguish directly adjudicated numerator/denominator counts from any estimated population totals. Estimates may use probability-sampled non-candidates with recorded inclusion probabilities only under an approved estimator and confidence method. Independently reported misses retain discovery/selection provenance but have no assumed inclusion probability or population weight unless they came from a documented probability sample. Do not present an opportunistic-miss count as a population estimate. Report abstention as completed assessments with an insufficient-evidence outcome divided by completed assessments in scope; provider or acquisition failures are not abstentions. For each required operation, report operationally incomplete evaluation examples divided by examples requiring that operation, including source resolution, native coverage, required assessment and recording. For every metric, state its numerator, denominator, exclusions, sample size, selection provenance and uncertainty, and do not hide failures by pooling unrelated scopes.
6. Assess KPI-01 on the independent holdout using the existing target of at least 95% precision among blocking findings in each qualified scope. Report true/false-positive counts and uncertainty. If there are zero predicted positive findings, precision is undefined. A sample too small to support the approved uncertainty method, or near-total abstention, cannot establish qualification or usefulness.
7. Before declaring any profile qualified, approve the sample-size/confidence method, per-profile recall and abstention floors, exact rule/language/project/analyzer/judge scope and prospective non-blocking comparison. These choices remain open and block qualification until approved. Performance workloads and limits remain governed separately by ADR-007; the performance reference repositories are not an adjudicated quality corpus by default.
8. Run malformed-output, partial-resolution, stale-generation, timeout, cancellation and recording/export-failure cases through the real public boundary. Keep independent labels separate from Jev's predictions. Requalify affected profiles when their evidence, analyzer, selector, renderer, model or relevant configuration changes, and complete the approved prospective non-blocking comparison before blocking activation.

#### Evaluation integrity example (L01.3)

- **V07 — Independent labels, lineage and qualification gates:** Jev supports a candidate and CI completes successfully, but an independent adjudicator rejects the issue; retain the provider prediction separately from the attributed adjudication. A reviewer suppresses another finding or defers a real issue; retain those dispositions as feedback without turning either into an independent positive or negative label. Two clones or related revisions retain their individual example identities and history while sharing one lineage partition, even when a deduplicated view is used. Freeze the profile-specific independent holdout before tuning selectors, prompts, thresholds or models. If predicted positives are absent, precision stays undefined; if sample/confidence criteria, recall or abstention floors, the exact profile, or the prospective comparison remain unapproved, the profile cannot qualify.

A qualification result identifies the exact validated scope and versions. It is not a repository-wide or language-wide quality guarantee. The first qualifying combination will be selected from the evidence, not guessed in this PRD.

## 4. Technical Specifications

### Architecture Overview

The shared Rust engine follows the ownership boundaries in [ADR-001](decisions/001-rust-engine-and-client-owned-sessions.md): explicit scope → captured inputs → native facts → candidates and evidence/counterevidence → semantic assessment where needed → policy → report. Durable records are collected throughout this flow, including the input and request before new inference is sent.

The CLI and resident transport invoke that engine; `output` prepares one projection for TOON/JSON. SQLite is the authoritative record store, while JSONL/evidence artifacts are derived exports. Language and model adapters isolate native/provider-specific types. Reusable facts and assessments retain source/configuration identity and cannot be published as current after invalidation or supersession.

Detailed resident methods, read-report exit behavior, source-view mechanics, database schema and codec profile remain implementation contracts to specify. The initial analyze exits are defined in US-03. These later contracts must satisfy the user-visible criteria above and cannot silently redefine them.

### Project Configuration

[ADR-008](decisions/008-project-configuration-and-schema.md) owns D23–D25: one root JSONC configuration with path overrides (O1), a published versioned `$schema` link for editing, and documented limits with a compatible evolution toward subproject configurations (O2). O2 is future work, not a first-release requirement.

Configuration support must provide schema-assisted editing and explicit validation errors, reproduce the same effective settings across CLI/CI/resident entrypoints, and preserve those settings and their provenance in run records without secrets. Runtime validation must work without fetching the schema. Configuration cannot supply an implicit analysis scope, grant blocking qualification or substitute for trusted transmission authorization. Qualification of multi-file findings must account for the applicable path overrides.

**Initial defaults selected on 2026-10-14:** if the selected workspace has no root `.anti-slop.jsonc`, use documented built-in defaults. A rule without an explicit setting defaults to advisory, not blocking. The effective view identifies built-in values and their origins; it does not claim an absent configuration file was read. An existing malformed or unreadable file remains an error rather than falling back to defaults. These defaults do not supply an analysis scope, establish capabilities or qualification, or authorize transmission. Exact setting fields and explicit configuration-path behavior remain OPEN-08 choices recorded in the current AS-M002 contract; this selection does not implement configuration support.

**Override and policy boundaries selected on 2026-10-14:** path overrides use explicit include/exclude glob lists, relative to the declaring configuration, rather than ordered gitignore-style patterns. Matching an override changes rule settings; it does not exclude source or context from analysis. Effective configuration inspection exposes each path's settings and origins. When a finding spans differently configured paths, combining those settings into finding policy remains a rule-implementation gate; AS-M002 supplies the individual views without selecting an aggregate policy. Exact glob grammar and the rule-specific attribution algorithms remain OPEN-08 qualification gates.

**Initial inspection and schema surface selected on 2026-10-14:** `config explain --workspace <dir> --path <path> [--config <file>]` provides TOON-default and explicit JSON inspection of the same effective configuration and origins. The initial fields are `$schema`, `rules` and ordered `overrides`; rule identifiers R1–R9 use scalar `off`, `advisory` and `block` policies. Requested policy does not implement or qualify a rule. Generate the local schema using draft 2020-12 and reserve versioned schema URLs under GitHub release assets in `abdwhb-png/anti-slop`. Exact local behavior, identity mapping, format flag and qualification checks belong to the active AS-M002 contract. Publication and verified public availability remain separate release gates; none of these surfaces is implemented by this documentation.

**AS-M002 implementation:** the shared resolver implements O1 defaults, ordered overrides and immutable provenance. `config explain` exposes TOON 4.1 and JSON inspection. The schema derives from the Rust input types and is checked locally through shared fixtures and the VS Code JSON language service. The [recovery record](../.missions/AS-M002/recovery/README.md) distinguishes the original acceptance from fresh checks and missing audit records. The schema asset remains unpublished. Multi-file finding policy, source capture and analysis retain their later gates.

### Integration Points

| Boundary | Required integration behavior |
| --- | --- |
| Git and repository inputs | Resolve the explicitly requested selection and base, preserve dirty/relevant untracked inputs, and disclose missing history or unstable capture. |
| Native analyzers | Consume a coherent source/configuration/dependency view; report capabilities, mapping quality and coverage. Use verified provisioning separately from normal analysis. |
| Model provider | Use configured credentials through secure runtime configuration, record identity and request/response provenance without credentials, and apply D22 and the total phase deadline. |
| Local SQLite | Own durable analyses/evidence/feedback; expose capture, migration, quota and export failures honestly. No hosted database is required. |
| CLI and CI | English machine-consumable reports, explicit scope, format equivalence and distinct policy/operational outcomes. Initial analyze commands, selectors and exits are defined in US-01/US-03; retained-report errors and future resident operations remain to specify. |
| Resident client | Client-owned lifetime, bounded operations and source-generation-aware reports. No specific future Pi event schedule is assumed. |

### Security & Privacy

- Local storage and explicit export are the collection model. Sending bounded evidence to a configured judge is a separate transmission, allowed only under the repository authorization in D22.
- Define source inclusion/exclusion and transmission controls before the first remote integration. Explicitly selecting changes must not silently bypass those controls. Never send a whole repository merely because some unchanged context is needed.
- Keep provider credentials out of retained transport records, debug logs, reports and export bundles. Use existing secure configuration channels; no credential-pasting workflow is required.
- Treat analyzed source as untrusted input to both the judge and native tooling. Declare any execution of build scripts, procedural macros or project helpers; the restricted research configuration is not proof that arbitrary project code is safe to execute.
- Preserve the working tree and isolate mutable analyzer state. Unsupported or unbound external inputs remain coverage limitations.
- Explicit purge and export must state their selected data and outcome. Do not claim deletion from previous exported copies or a remote provider when only local records were purged.

No jurisdiction-specific certification or provider retention guarantee has been selected. Provider data-use terms and the exact credential/execution controls must be established for the actual deployment before making such claims.

## 5. Risks & Roadmap

### Phased Rollout

The [progressive implementation roadmap](IMPLEMENTATION_ROADMAP.md) sequences the accepted first-release scope, starting with the user-selected TypeScript analysis path. It preserves the original lot plan, proposes the next wave for approval and details only the next Mission Contract. It assigns the open gates below to their owners without changing release acceptance or claiming the planned work is complete.

| Phase | Deliverable and exit conditions |
| --- | --- |
| Internal qualification slice | One complete source/evidence/assessment/policy/record/report/feedback/export path, including a justified-adapter counterexample and incomplete/failure cases. Validate real ownership boundaries. This is not a reduced-scope first release. |
| MVP / first usable release | R1–R9 paths in all three languages; Linux/WSL2; explicit scopes; CLI and client-owned residency; configured judge plus mode without model calls; durable collection, feedback and portable export. Pass the applicable KPIs and qualify at least one rule/language combination for optional selective blocking. |
| v1.1 — Proposed follow-up | Extend qualified scopes and improve measured accuracy/resource use. Design the Pi integration against the implemented resident contract and measure its actual hook overhead before claiming readiness. |
| v2.0 — Conditional direction | Integrate a better judge when an independently evaluated candidate exists. Reconsider other native adapters, platforms or Trellis only with new evidence and explicit scope decisions. No model identity, launch date or platform expansion is promised. |

The provider-neutral boundary is part of the MVP; only adoption of an actual future replacement is conditional. No delivery deadline, staffing assumption or monetary inference budget has been selected.

### Technical Risks

| Risk | Consequence | Required response or gate |
| --- | --- | --- |
| RISK-01 — Native startup and retained memory exceed targets | Interactive use and the future Pi integration become impractical. | Measure the full path and owned process tree; improve or replace an adapter, or explicitly revise the target. |
| RISK-02 — Stale, partial or incorrectly mapped facts | Plausible findings cite the wrong code or infer absence incorrectly. | Qualify source generations, overlays, configuration/dependencies and late responses; preserve coverage/mapping limits. |
| RISK-03 — Small or contaminated evaluation sets | A blocking profile appears precise but fails on real work. | Independent adjudication, frozen lineage-aware splits and approved sample/confidence and recall/abstention gates. |
| RISK-04 — Candidate selection misses defects | High precision conceals low usefulness. | Evaluate candidate and end-to-end recall using sampled non-candidates and manually reported misses. |
| RISK-05 — Source disclosure or uncertain remote attempts | Unintended data transfer or duplicate inference cost. | Explicit repository authorization, bounded source selection, credential exclusion and preserved attempt/cost uncertainty. |
| RISK-06 — Storage exhaustion or failed durability | Training inputs disappear or acknowledged feedback cannot be recovered. | Admission accounting, explicit failure states, crash/recovery and export-closure tests under ADR-005. |
| RISK-07 — Native/API or model drift | Fact contracts and blocking qualification silently become invalid. | Record effective identities, test adapter contracts and requalify affected profiles. |

### Open Specifications and Release Gates

| Item | Remaining choice | Resolve before |
| --- | --- | --- |
| OPEN-01 | The initial CLI commands, four analysis selectors, direct base comparison and analysis exit values are resolved under US-01/US-03. The public compatibility policy for later CLI/protocol revisions, error outcomes for read-only report requests and method names/semantics for the future resident API remain open. D19 settles that every analysis request requires an explicit scope. | Publishing versioned CLI or resident-protocol compatibility guarantees. |
| OPEN-02 | **Established protocol:** independent, attributed labels remain distinct from provider predictions, feedback, suppression, deferral, insufficient evidence and failures; related repository/revision/clone lineage stays in one immutable partition; a profile-specific independent holdout is frozen before tuning; non-candidate samples and independently found misses retain selection provenance; candidate recall, assessment precision, end-to-end recall, abstention and operational incompleteness are reported separately with counts, denominators, exclusions and uncertainty. **Before corpus construction or tuning:** declare the target population, representative project characteristics, corpus composition, label/adjudication protocol and revision, immutable partition plan, and sampling/selection provenance. **Before labels are assigned:** explicitly select and approve adjudicator identities and the disagreement-resolution process. **Before qualification:** approve the population-estimation and sample-size/confidence methods, per-profile recall and abstention floors, exact rule/language/project/analyzer/judge scope, and prospective non-blocking comparison. No first qualifying rule/language or numeric sampling/recall/abstention criterion is selected here. D20/D21 keep KPI-01 at at least 95% blocking precision and require at least one qualified combination for the first usable release. | Declare the population, representation, corpus and labeling/partition plan before corpus construction or tuning; approve the adjudicator process before assigning labels; approve the named qualification gates and complete the prospective non-blocking comparison before qualification or activation. Those unresolved gates block qualification and shipping the first usable release. |
| OPEN-03 | Provider/version and endpoint setup, credential handling, repository transmission authorization, source exclusions, native code-execution controls and monetary budgets if required. D22 settles invocation behavior after configuration/authorization. | Real remote inference or executing project-supplied native code. |
| OPEN-04 | Source-view/cache mechanisms, Linux/WSL2 descendant cleanup and effective whole-tree accounting. | Production adapter/session acceptance and resource claims. |
| OPEN-05 | SQLite schema, transactions/journaling, quota reservations, export layout and purge/recovery mechanics. | Durable collection/export acceptance. |
| OPEN-06 | TOON codec, declared specification profile, default detail limits and report schema/versioning. | Public report interoperability claims. |
| OPEN-07 | Actual replacement model and training interface; Pi triggers and delivery cadence; later platforms. | Their future milestones, not the MVP's replaceable core boundary. |
| OPEN-08 | Exact configuration fields and resolution, multi-file policy attribution, JSONC parser, schema publication URL and schema/runtime/editor conformance under ADR-008. D23–D25 settle the format, initial organization and evolution requirements. | Publishing O1 configuration support. O2 composition and migration tooling are qualified in a later O2 task. |

Resolve mission work through the current Mission Contract and workflow linked by the [roadmap](IMPLEMENTATION_ROADMAP.md); retain the [bounded task template](templates/task.md) for standalone work. A completed task establishes its own scope; it does not waive an unresolved release gate or reduce the agreed language/rule coverage.
