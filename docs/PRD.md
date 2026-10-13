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

- Support an explicitly selected local-change scope and a comparison using an explicitly supplied Git base. Define their exact selectors and comparison semantics in the public contract before implementation.
- A missing, invalid or unresolved selector produces a clear usage/configuration failure before starting analyzers or model calls. Do not fall back to the current branch, the previous analysis or a configured default silently.
- Resolve and report the effective selection, comparison base where applicable, and captured source identity. Local selection must distinguish index, working-tree and relevant untracked inputs according to the requested scope.
- Keep changed-code selection distinct from supporting repository context. Relevant unchanged callers, contracts and history may be gathered as evidence without claiming they were all selected changes.
- Preserve the working tree. Detect unstable acquisition and state the achieved consistency/freshness scope; do not treat a commit hash alone as the complete local input identity.
- Help and report-reading operations require no analysis scope because they do not analyze code.

#### US-02 — Understand findings and uncertainty

As a developer, I want evidence and counterevidence for each finding so that I can decide whether the code should change.

- Identify the rule, captured source location where accurately mapped, actual observations, attributed assessment and relevant counterevidence. Missing or approximate mappings remain explicit.
- Provide bounded report views with truthful counts, truncation and access to further detail. TOON and explicit JSON must express the same selected report under [ADR-006](decisions/006-output-formats-and-shared-projection.md).
- Distinguish execution, evidence coverage, assessment, freshness, policy and recording state. No findings, rejected candidates, insufficient evidence and unavailable execution are different outcomes.
- Group related findings about one actionable issue without erasing their rule identities or inflating issue counts. Display filters do not change CI policy.
- A useful thin adapter or necessary comment must remain a legitimate counterexample. Provider confidence or explanatory prose is not additional extracted evidence.

#### US-03 — Enforce qualified CI policy

As a repository maintainer, I want selected rules to block only within their validated scope so that CI provides a trustworthy design gate.

- Require an explicit analysis scope and comparison base when the selected CI mode needs one; CI environment metadata must not silently choose another scope.
- Allow blocking only for an enabled rule whose qualification profile matches the rule/candidate selector, language/project characteristics, analyzer/evidence contract, effective judge/renderer and required coverage.
- Make at least one qualified combination available in the first release, subject to KPI-01 and the qualification procedure in Section 3. Shipping that profile does not automatically enable it in every repository.
- Preserve separate public outcomes for qualified violations and inability to complete required work. Required analyzer/judge/recording failures must not report a complete clean check; advisory incompleteness remains visible even when it does not fail the job.
- If a violation and operational failure coexist, retain both in the report. Exact process exit values belong to the public CLI contract and remain to be specified.
- Produce self-contained CI artifacts when export is requested. An artifact or recording failure cannot be reported as a complete dataset capture.

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

- Attach feedback to an exact finding/evidence occurrence and record actor kind, disposition, optional rationale and adjudication state.
- Preserve false-positive reports, confirmed issues, suppression, accepted trade-offs and deferred work distinctly. Agent feedback is not silently relabeled as human adjudication.
- Correct feedback through an attributed revision that preserves its history. A durable acknowledgement follows persistence, not merely queuing.
- Support reporting a missed issue, with its source context, so evaluation is not limited to candidates the system already selected.
- Materially changed evidence does not silently inherit an adjudicated label from an earlier occurrence. A reviewed carry-forward mechanism needs explicit provenance.

#### US-07 — Export, aggregate and evaluate independently

As a dataset curator, I want complete portable exports so that local and CI examples can be aggregated and evaluated without their original machines.

- Export versioned JSONL with the evidence required to resolve every included example. Verify completeness after copying the artifact away from its originating workspace and database.
- Preserve provenance, distinct dispositions, feedback revisions, transformation identity and split membership. Do not turn an incomplete example into a negative label.
- Aggregate explicitly and preserve repository/code lineage while deduplicating related examples. Training and evaluation separation must account for clones, related revisions and time.
- Freeze an independently adjudicated holdout before tuning. Performance repositories are not automatically a labeled quality corpus.
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

1. Build positive, legitimate-negative and ambiguous fixtures for every R1–R9/language combination. Exercise actual native facts and the public engine path; use controlled provider responses only when testing the surrounding contract.
2. Record candidate-generator recall separately from assessment precision, end-to-end recall, abstention and operational incompleteness. Include sampled non-candidates and independently found misses in the evaluation population.
3. Establish a reviewed labeling/adjudication protocol, representative project scopes and lineage-aware partitions. Freeze the holdout before tuning candidate selectors, prompts, thresholds or models.
4. Select thresholds on validation data and assess KPI-01 on the independent holdout. Publish true/false-positive counts, sample sizes, intervals/uncertainty and excluded cases. Do not pool unrelated scopes to hide a failing rule/language combination.
5. Before declaring a profile qualified, approve its sample-size/confidence protocol and recall/abstention acceptance gates. Zero predicted positives leaves precision undefined; abstaining on nearly everything is not evidence of a useful detector. These details remain open and block qualification until resolved.
6. Run the malformed-output, partial-resolution, stale-generation, timeout, cancellation and recording/export-failure cases through the real public boundary. Keep adjudicated labels independent of Jev's predictions.
7. Requalify affected profiles when their evidence, analyzer, selector, renderer, model or relevant configuration changes. Run a prospective blocking configuration in non-blocking comparison before activation.

A qualification result identifies the exact validated scope and versions. It is not a repository-wide or language-wide quality guarantee. The first qualifying combination will be selected from the evidence, not guessed in this PRD.

## 4. Technical Specifications

### Architecture Overview

The shared Rust engine follows the ownership boundaries in [ADR-001](decisions/001-rust-engine-and-client-owned-sessions.md): explicit scope → captured inputs → native facts → candidates and evidence/counterevidence → semantic assessment where needed → policy → report. Durable records are collected throughout this flow, including the input and request before new inference is sent.

The CLI and resident transport invoke that engine; `output` prepares one projection for TOON/JSON. SQLite is the authoritative record store, while JSONL/evidence artifacts are derived exports. Language and model adapters isolate native/provider-specific types. Reusable facts and assessments retain source/configuration identity and cannot be published as current after invalidation or supersession.

Detailed wire methods, exit values, source-view mechanics, database schema and codec profile are implementation contracts still to specify. They must satisfy the user-visible criteria above and cannot silently redefine them.

### Project Configuration

[ADR-008](decisions/008-project-configuration-and-schema.md) owns D23–D25: one root JSONC configuration with path overrides (O1), a published versioned `$schema` link for editing, and documented limits with a compatible evolution toward subproject configurations (O2). O2 is future work, not a first-release requirement.

Configuration support must provide schema-assisted editing and explicit validation errors, reproduce the same effective settings across CLI/CI/resident entrypoints, and preserve those settings and their provenance in run records without secrets. Runtime validation must work without fetching the schema. Configuration cannot supply an implicit analysis scope, grant blocking qualification or substitute for trusted transmission authorization. Qualification of multi-file findings must account for the applicable path overrides.

### Integration Points

| Boundary | Required integration behavior |
| --- | --- |
| Git and repository inputs | Resolve the explicitly requested selection and base, preserve dirty/relevant untracked inputs, and disclose missing history or unstable capture. |
| Native analyzers | Consume a coherent source/configuration/dependency view; report capabilities, mapping quality and coverage. Use verified provisioning separately from normal analysis. |
| Model provider | Use configured credentials through secure runtime configuration, record identity and request/response provenance without credentials, and apply D22 and the total phase deadline. |
| Local SQLite | Own durable analyses/evidence/feedback; expose capture, migration, quota and export failures honestly. No hosted database is required. |
| CLI and CI | English machine-consumable reports, explicit scope, format equivalence and distinct policy/operational outcomes. Exact command names and codes remain to specify. |
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
| OPEN-01 | Exact local-change selectors, Git-base comparison semantics, command/method names, compatibility rules and process exit values. D19 already settles that a scope is mandatory. | Implementing/publishing the corresponding public contract. |
| OPEN-02 | First qualifying combination, adjudicators, corpus composition, sample sizes/confidence method, recall/abstention floors and representative project characteristics. D20/D21 settle the minimum release qualification and precision target. | Declaring a profile qualified or shipping the first usable release. |
| OPEN-03 | Provider/version and endpoint setup, credential handling, repository transmission authorization, source exclusions, native code-execution controls and monetary budgets if required. D22 settles invocation behavior after configuration/authorization. | Real remote inference or executing project-supplied native code. |
| OPEN-04 | Source-view/cache mechanisms, Linux/WSL2 descendant cleanup and effective whole-tree accounting. | Production adapter/session acceptance and resource claims. |
| OPEN-05 | SQLite schema, transactions/journaling, quota reservations, export layout and purge/recovery mechanics. | Durable collection/export acceptance. |
| OPEN-06 | TOON codec, declared specification profile, default detail limits and report schema/versioning. | Public report interoperability claims. |
| OPEN-07 | Actual replacement model and training interface; Pi triggers and delivery cadence; later platforms. | Their future milestones, not the MVP's replaceable core boundary. |
| OPEN-08 | Exact configuration fields and resolution, multi-file policy attribution, JSONC parser, schema publication URL and schema/runtime/editor conformance under ADR-008. D23–D25 settle the format, initial organization and evolution requirements. | Publishing O1 configuration support. O2 composition and migration tooling are qualified in a later O2 task. |

Resolve mission work through the current Mission Contract and workflow linked by the [roadmap](IMPLEMENTATION_ROADMAP.md); retain the [bounded task template](templates/task.md) for standalone work. A completed task establishes its own scope; it does not waive an unresolved release gate or reduce the agreed language/rule coverage.
