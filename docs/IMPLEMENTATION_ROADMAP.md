---
{
  "schemaVersion": 1,
  "kind": "roadmap",
  "id": "anti-slop-v1",
  "revision": 1,
  "refs": [
    { "kind": "document", "path": "PRD.md" },
    { "kind": "document", "path": "decisions/README.md" },
    { "kind": "document", "path": "../README.md" },
    { "kind": "mission", "id": "AS-M001", "revision": 2, "path": "missions/AS-M001/r2.md" },
    { "kind": "evidence", "path": "../.missions/AS-M001/evidence/preparation-2026-09-23.md" }
  ]
}
---

# Anti-Slop progressive implementation roadmap

Prepared: 2026-09-23. Updated: 2026-10-13. **AS-M001 documentation execution and sequential fresh-context agents are authorized by the user-approved plan. W01 as a whole and later mission execution remain proposed. No commit or publication is authorized.**

This is the existing roadmap's first workflow revision. It retains the complete 2026-09-22 plan [below](#retained-2026-09-22-plan), unchanged, as historical planning context. The active view above that history owns current sequencing. Historical subtask prompts are not current dispatch instructions. No PRD, ADR, research record or roadmap file has been replaced or moved.

## Sources and workflow

| Information | Authoritative owner |
| --- | --- |
| Product behavior, D1–D25, US-01–US-09, KPIs and unresolved specifications | [Existing PRD](PRD.md) |
| Accepted architecture and its rationale | [Existing ADRs](decisions/README.md) |
| Implemented capabilities | [README current status](../README.md#current-status), checked against source |
| Destination, dependencies and the next approval frame | This roadmap |
| One bounded outcome and its validation/authority | [AS-M001 revision 2](missions/AS-M001/r2.md), currently the only detailed Mission Contract |
| Starting evidence and resumable mission state | [Preparation evidence](../.missions/AS-M001/evidence/preparation-2026-09-23.md), [state](../.missions/AS-M001/state.json) and append-only [history](../.missions/AS-M001/history.jsonl) |

Use the repository-installed [mission-driven-dev skill](../.agents/skills/mission-driven-dev/SKILL.md) for phase instructions and artifact formats. Historical preparation used a sibling checkout; current execution uses this installed copy. Do not substitute the retained generic prompt if it is unavailable. The [standalone task template](templates/task.md) remains available for work outside a mission; Task Packets are derived from an approved contract just before authorized dispatch.

## Destination and starting evidence

The accepted destination remains the first usable Linux/WSL2 release described by the PRD: R1–R9 across TypeScript, PHP and Rust; explicit scopes; root `.anti-slop.jsonc` and a linked schema; one shared engine for CLI/CI and client-owned residency; retained evidence, feedback and portable datasets; replaceable judges; and at least one independently qualified rule/language combination for selective blocking. Product surfaces stay English.

**TypeScript is the user-selected first complete analysis path.** Use controlled fixtures before the initial real-repository corpus. R1 remains the earlier proposed narrow pilot, not an approved first blocking combination. A pilot does not reduce the release matrix or waive its quality/resource gates.

The historical preparation inspected `main` at `0168328e1bd1328bd2ca613e3840656367fca3e7`, with the dirty state retained in preparation evidence. Current execution uses clean base `4f16dca764a6037176f5796e50d8e25db523dd92` in `/tmp/anti-slop-as-m001`, as recorded in authorization evidence. Source and CLI test inspection show help/version and rejection of unavailable analysis only; reserved directories are not working modules. No application tests, analyzers or corpus runs were executed during this preparation.

O2 configuration composition, Pi packaging/hooks, later operating systems, Trellis, hosted collection, automatic rewriting and training/deploying a replacement model stay outside v1. Preserve the accepted extensibility boundaries without implementing those future products.

## Next approval frame: W01

**Wave decision pending:** AS-M001 alone has execution authority in its [authorization evidence](../.missions/AS-M001/evidence/authorization-2026-10-13.md). Approve or revise the remaining wave outcomes separately. Only AS-M001 has a detailed contract. Each eventual executable mission concerns the Anti-Slop repository alone. Approval of this frame does not resolve a substantive open product choice or authorize launch, delegation, commits, installation, external transmission or publication.

| Mission | Observable outcome and dependency | Boundary | Exit evidence required |
| --- | --- | --- | --- |
| **AS-M001 — Initial analysis contracts** | The initial TypeScript path has an unambiguous request/outcome contract and an evaluation protocol sufficient to begin the next implementation contracts. Existing L00 is the input; covers L01.1–L01.3. | Documentation in existing owners; begin with L01.1. Define only the identities/status distinctions needed by the pilot and the evaluation protocol/gates. No production interfaces, dataset labeling campaign or implementation. | Concrete valid, missing-scope, counterexample, partial/failure and recording examples; explicit identity/provenance obligations; independent evaluation and holdout gates assigned before tuning/qualification; product choices recorded or dependent work explicitly blocked. |
| **AS-M002 — Effective root configuration** | One inspectable effective `.anti-slop.jsonc` configuration resolves and validates under ADR-008. Depends on accepted AS-M001; maps to L02. | O1 root configuration, ordered overrides, offline validation and local schema generation/conformance. Resolve the relevant OPEN-08 choices first. No O2, analyzer/model work, public hosting or dependency installation without the applicable authority. | Focused behavior tests cover overrides, origins, path bases and invalid/ambiguous JSONC; schema/runtime/editor agreement is demonstrated locally. No implicit scope or transmission permission. Unpublished schema status is explicit. |
| **AS-M003 — Coherent selected sources** | Explicitly selected changes and supporting source are captured with reproducible identities. Depends on accepted AS-M001 and AS-M002; maps to L03. | Controlled local/Git repositories and bounded source acquisition without working-tree mutation. No native adapter, model, storage system or resident implementation. | Public-boundary tests cover index/worktree/untracked distinctions, additions/deletions/renames, invalid bases, exclusions and unstable reads; spans bind to actual bytes. Native overlay acceptance remains a later adapter gate. |

W01 unlocks configuration and source capture, not a successful analysis command. AS-M001 may refine examples within accepted decisions; a new product trade-off still needs a recorded user decision. Do not launch AS-M002/AS-M003. AS-M001 packets are created only immediately before authorized dispatch.

## Later outcomes and dependencies

These rows preserve the original lot coverage. They are an outcome map, not detailed missions or blanket authorization; split a row into bounded missions when its wave becomes next.

| Retained lot coverage | Observable outcome | Dependency and proof that permits progress |
| --- | --- | --- |
| L04 → L05 | Evidence is durable and retained reports expose truthful bounded TOON/JSON views. | W01; persistence, recovery, admission and equivalent report states pass their public boundaries before inference. Report reads do not trigger analysis. |
| L06 → L07 | The first complete TypeScript one-shot analysis retains native evidence, a replaceable assessment and its report. | L03–L05; source binding, legitimate counterexamples and explicit failure/abstention paths. Resolve actual provider/execution/transmission controls before any corresponding external action. |
| L08 | Attributed feedback and portable exports can be independently reconstructed and explicitly purged. | L07; records retain actual evidence, label provenance and export closure. This completes the initial learning-data path. |
| L09 | A client-owned resident session reuses the same engine and releases its owned processes. | L07 with stable store/report contracts; cancellation, disconnect, stale responses and process cleanup are demonstrated. L08 and L09 may be planned separately; no parallel delegation is currently authorized. |
| L10; L11 | PHP and a separately bounded Rust integration pass the shared analysis/data/session contract. | L08 and L09; each adapter supplies its own coherence, capabilities and whole-process resource evidence. These remain separate repository-local missions, not one multi-adapter assignment. |
| L12 | All R1–R9/language paths have positive, legitimate-negative and ambiguous evidence cases. | TypeScript after L08; PHP after L10; Rust after L11. Plan individual rule/language outcomes as the adapters become available; unsupported-only cells do not count as delivered. |
| L13 | At least one profile meets the PRD qualification gates and selective CI policy works. | L08 and L12; independent frozen evaluation, the accepted precision target, approved statistical/recall criteria and failure handling. The first blocking combination remains an evidence-based choice. |
| L14 | Integrated resource and failure checks meet release gates. | L09–L13; measure the complete ADR-007 boundaries on recorded representative workloads, including mixed use and failure recovery. Early measurements remain required as each path is implemented. |
| L15 | An installable release candidate, usable public schema and operations documentation are reviewable. | L02, L08, L13 and L14; actual schema availability, compatibility, install and data-reconstruction evidence. Public release still requires its own authority. |

L01's evaluation work is not postponed to L13: AS-M001 establishes the protocol and gates; corpus construction grows with rule work; freeze each independent holdout before tuning its profile. Sample/confidence and recall/abstention choices must be settled before a qualification claim. Release acceptance remains in the PRD, including all nine stories and five KPIs; the retained traceability table remains a navigation aid.

## Contradictions, gaps and their owners

| Reference | Observation | Resolution or pending decision |
| --- | --- | --- |
| C01 | The earlier roadmap detailed distant work; the selected workflow progressively details only the next mission. | Keep all previous text unchanged below as history; use this outcome view and W01 for future elaboration. Historical task lists cannot authorize dispatch. |
| C02 | The earlier first-assignment prompt and general task-template routing do not supply Mission Contract/state/history semantics. | Keep the prompt/template for history and standalone work; route mission work through AS-M001 and the supplied skill. Create Task Packets only at authorized dispatch. |
| C03 | The scaffold has a TOML dependency while accepted project configuration is JSONC. This is already explained in the README. | Preserve the scaffold now. AS-M002 reviews the actual dependency change under the existing supply-chain workflow; no competing TOML product contract is introduced. |
| G01 | AS-M001 documentation execution and sequential fresh authors/validation are authorized. W01 as a whole and later missions remain unapproved. | Follow r2 and authorization evidence. No commit, installation, integration or publication is implied. |
| G02 | The PRD now specifies initial analyze/report operations, exact selectors, direct-base comparison, analysis exits and minimal provenance/status obligations. | The mission state owns independent acceptance. Later CLI/protocol compatibility, report-read failures and detailed transport/serialization remain at their owning gates. No command is implemented. |
| G03 | The PRD now defines independent labels, lineage partitions, holdout-before-tuning and distinct metrics. OPEN-02 still gates corpus/adjudicator choices, statistics, recall/abstention floors and exact profiles. R1 is only a pilot proposal. | Follow the established protocol and unresolved gates. No corpus is labeled or profile qualified by this documentary result. |
| G04 | OPEN-08 leaves configuration details, parser and the real public schema URL unresolved. | AS-M002 must settle its local contract and state publication limits; verify the chosen real schema endpoint before public support/release. Do not invent a URL or alter ADR-008. |
| G05 | OPEN-03–OPEN-06 include provider/execution controls, native views, storage, report encoding and session accounting. | Resolve just before the owning L03–L09 contract or external action needs them; prior probes do not settle production behavior. OPEN-07 and O2 stay future work. |

No conflict between accepted product decisions was found in this preparation. The issues above distinguish workflow mismatches, a known scaffold gap and genuinely open specifications; they are not permission to reopen approved architecture.

## Reassessment and records

At each accepted mission boundary, check actual behavior, retained evidence, unresolved gates and downstream dependencies before detailing the next contract. Stop earlier if a contradiction changes scope, constraints, authority, outcome or validation. Keep approved revisions and completed history; changed approved intent requires a new revision and reassessment of affected work. The unapproved wave does not freeze speculative future task details.

The active Mission Contract and `.missions/AS-M001/` describe documentation execution and its acceptance evidence, not Anti-Slop's future product dataset. Keep these auditable records separate from ignored `.anti-slop/` application data. The preserved initial state had no packet, attempt, result or checkpoint. Current state and history track authorized execution; no commit checkpoints are permitted. Structural checks establish artifact consistency only; they do not establish readiness, independent acceptance or permission to run a mission.

## Retained 2026-09-22 plan

The following original text is retained verbatim in this same file. Its execution status, detailed distant tasks and suggested prompt describe the earlier planning state; use the progressive view above for current workflow and authority. No historical task has been performed merely by preserving it.

<details>
<summary>Original lot-by-lot plan and traceability (historical)</summary>

# Anti-Slop first-release implementation roadmap

Date: 2026-09-22

Status: **Execution plan for the accepted first-release scope.** L00 is present; L01–L15 are planned. The user selected TypeScript for the first complete analysis path. This document sequences implementation and its gates; it does not implement features, declare qualification, replace accepted decisions or authorize external publication.

The [PRD](PRD.md) owns release acceptance, the [ADR index](decisions/README.md) owns architectural decisions, and the [root README](../README.md#current-status) owns implemented state. This roadmap owns sequencing. Use the [task template](templates/task.md) for each bounded implementation assignment.

## Destination and verified starting point

Deliver the first usable Linux/WSL2 release with R1–R9 paths in TypeScript, PHP and Rust, explicit analysis scopes, root `.anti-slop.jsonc` configuration with a linked schema, one-shot and client-owned resident operation, durable evidence/feedback, portable exports, replaceable judges and at least one qualified rule/language combination available for selective CI blocking. Product surfaces remain English.

Inspection of `src/cli.rs`, `src/main.rs`, `src/lib.rs` and `tests/cli.rs` confirms only help/version and rejection of unavailable analysis. Module directories are reserved, not implementations. Installed dependencies and historical experiments are starting material, not completed integrations. Preserve existing documentation edits when starting a task.

Contracts still open in the PRD are assigned to their owning lots below. Schema hosting, exact CLI/selectors, native execution controls, evaluation statistics and source/store/session contracts are not silently settled by this schedule.

## Delivery approach

| Approach | Fit and cost | Limitation and reversibility |
| --- | --- | --- |
| S3 — Shared vertical path, then language/rule expansion (recommended structural alternative) | Exercise capture, records, native evidence, assessment and reports together early; extend the same boundaries afterward. | Keep the pilot explicitly narrower than v1. Other languages may require evidence-contract changes; investigate their integration risks before broad rule expansion. |
| S1 — Complete each technical layer first (conventional baseline) | Straightforward module ownership and little initial integration work. | Delays cross-layer failures and encourages unused interfaces. Switching later can affect several completed layers together. |
| S2 — Complete one language before starting others (conventional delivery alternative) | Produces deep coverage in one ecosystem early. | Defers PHP/Rust capability/resource risks and can entrench language-specific assumptions. Extraction boundaries permit reversal but do not eliminate its cost. |

Use S3 with **TypeScript first, as selected by the user**: controlled fixtures before Pi-lens. R1 is the proposed narrow pilot because it exercises native source context, semantic judgment and legitimate counterexamples. This sequencing does not predict which combination will first qualify for blocking. Integrate the other languages before completing the full rule matrix.

## Lots and dependencies

Dependencies name completed integration prerequisites unless a row permits work by language. A lot is a milestone containing bounded tasks, not a request for one agent to implement everything in one turn. No calendar estimate is asserted without throughput and qualification evidence.

| Lot | Observable outcome | Prerequisites |
| --- | --- | --- |
| L00 | Existing Rust scaffold and project instructions. | Present; preserve it. |
| L01 | Initial executable contracts and evaluation protocol are reviewable. | L00 |
| L02 | Root JSONC configuration resolves and validates consistently. | L01 |
| L03 | Explicit changes and supporting source are captured coherently. | L01, L02 |
| L04 | Evidence and execution attempts have durable local records. | L03 |
| L05 | Stored reports have truthful bounded TOON/JSON views. | L04 |
| L06 | TypeScript native evidence and a pilot R1 candidate path work. | L03, L04, L05 |
| L07 | First one-shot analysis uses the replaceable judge and records its outcome. | L06 |
| L08 | Feedback, portable exports, aggregation and explicit purge are usable. | L07 |
| L09 | A client-owned resident session reuses the same engine safely. | L07 |
| L10 | PHP passes the common analysis/data/session path. | L08, L09 |
| L11 | Rust passes the common analysis/data/session path. | L08, L09 |
| L12 | All R1–R9/language evidence and assessment paths are implemented. | TypeScript after L08; PHP cells after L10; Rust cells after L11. |
| L13 | At least one measured profile qualifies and selective CI policy works. | L08, L12 |
| L14 | Integrated resource and failure qualification meets release gates. | L09, L10, L11, L12, L13 |
| L15 | An installable release candidate, schema and operational documentation are ready. | L02, L08, L13, L14 |

L08 and L09 can progress separately once L07's record/report contracts are stable. L10 and L11 can be separate adapter lanes after their prerequisites. Rule work can progress per language as its real adapter becomes available. Parallel assignments require disjoint ownership and integrated validation; changes to shared generations, records or outcomes must be coordinated. This document does not launch agents or create tasks in another harness.

## L00: Existing scaffold

**Present:** one Cargo package, library/binary entrypoints, locked initial dependencies, CLI integration tests, project instructions, ADRs, PRD and repository skills. The CLI has no analysis feature.

**Use:** inspect the checkout and current edits before L01. Earlier build/test results are historical; preparing this roadmap does not rerun or certify them. Do not redo scaffolding or preinstall deferred dependencies.

## L01: Initial contracts and evaluation protocol

**Outcome:** the next implementation task has an observable public boundary and knows its unresolved decisions. Owners: `cli`, `workspace`, `engine`, `records`; sources: PRD OPEN-01/OPEN-02 and ADR-001/003/005/006/008.

**Bounded tasks:** L01.1 defines the first request/CLI operations, explicit local/Git-base selection semantics, invalid-input behavior and policy versus operational failure. L01.2 defines the minimal run/evidence/configuration identities and separate execution, freshness, coverage, assessment, policy and recording states needed by the pilot. L01.3 specifies the quality-corpus protocol, independent labeling, lineage-aware partitions and protection of held-out data before tuning.

**Exit:** concrete contract examples cover a legitimate counterexample, partial execution, a missing selector and failed recording, each with an unambiguous outcome. Incomplete evidence cannot become a clean required check. Identify product trade-offs requiring a user decision instead of inventing them. Defer detailed provider, transport and database choices to their lots; do not create unused Rust interfaces or a universal AST.

**Evaluation gate:** decide sample/confidence and recall/abstention acceptance criteria before qualifying a profile. Freeze each profile's holdout before tuning it. Corpus preparation begins here and grows with rule work; it is not postponed until L13.

## L02: JSONC configuration and schema

**Outcome:** the selected root `.anti-slop.jsonc` produces one validated, inspectable effective configuration. Owners: shared configuration module and CLI projection; sources: ADR-008 and PRD OPEN-08.

**Bounded tasks:** L02.1 finalizes initial fields, discovery/explicit-path behavior, rule values and precedence. L02.2 implements JSONC parsing, root overrides and effective-value origins. L02.3 generates JSON Schema from the authoritative contract and qualifies editor/runtime agreement, comments, trailing commas, duplicate keys, unknown settings and offline runtime loading.

**Exit:** overlapping overrides, relative paths and invocation from another directory resolve predictably; invalid settings produce located errors before analyzer/model work. No default scope, inferred transmission permission or nested inheritance is introduced. O2 remains deferred under ADR-008.

**Publication boundary:** choose the schema origin and version/compatibility mapping before advertising configuration support. Local generation/editor qualification can precede hosting; public `$schema` availability is verified in L15. Do not distribute a fictitious working URL. Review the historical TOML dependency and qualify only the JSONC/schema dependencies this lot needs.

## L03: Explicit scope and source generations

**Outcome:** capture the requested changes and supporting context without mutating the working tree. Owner: `workspace`; sources: PRD US-01 and ADR-002/008.

**Bounded tasks:** L03.1 implements the L01 local/Git-base selectors with index/worktree/untracked distinctions, additions, deletions and renames. L03.2 captures source/configuration/dependency identities and binds spans to actual bytes. L03.3 exercises unstable reads, exclusions, path boundaries and the managed-view strategy under controlled concurrent changes.

**Exit:** a missing/invalid base fails before native or model execution; selected changes stay distinct from unchanged evidence. Unstable acquisition is resolved within bounds or reported explicitly. Commit hashes/timestamps alone cannot establish coherence. Native-overlay binding is subsequently tested through each adapter, not declared solved here.

**Measurement:** record capture/invalidation cost with file/byte counts. Start with temporary controlled repositories; use frozen real-repository captures in later qualification tasks.

## L04: Durable evidence and admission

**Outcome:** local SQLite retains actual evidence and attributed run/attempt state before remote assessment is enabled. Owner: `records`; sources: ADR-005, PRD US-05/US-09 and OPEN-05.

**Bounded tasks:** L04.1 specifies schema, migrations, evidence placement and transactional acknowledgements. L04.2 implements run/evidence/attempt persistence and interrupted-write recovery. L04.3 implements accounting and admission reservations, including SQLite auxiliary files and terminal-status capacity, under the accepted retention/quota policy.

**Exit:** records reconstruct source bytes and identities after the workspace changes. Missing/corrupt evidence, read-only storage and interrupted writes remain failures. Admission failure rejects collection-dependent work before new model calls; authoritative evidence is not deleted automatically. Reads of retained data remain possible at the collection threshold. Complete exports arrive in L08.

**Measurement:** track persistence, admitted bytes, database growth and timing from this lot onward. Technical logs are not a second dataset writer. Use real temporary SQLite databases for behavior checks.

## L05: Bounded reports and truthful outcomes

**Outcome:** read retained run state and expose one projection through default TOON and explicit JSON. Owner: `output` and a shared report-read operation; sources: ADR-006, PRD US-02 and OPEN-06.

**Bounded tasks:** L05.1 freezes the report envelope, status axes, detail selection and outcome/exit mapping for the first CLI path. L05.2 qualifies the TOON profile/codec and both encodings. L05.3 implements bounded inspection with counts, truncation and source/configuration provenance.

**Exit:** empty, rejected, stale, partial, failed and recording-incomplete cases remain distinguishable. Unicode, multiline/control characters and numbers preserve semantics across encodings. Display filtering cannot hide policy failure. Stored-report reads cannot trigger capture, native analysis or inference.

**Boundary:** controlled retained fixtures exercise the real report/store boundary; they do not justify a successful analysis command before L07. Measure serialization/store-read cost now; B1's complete resident boundary becomes measurable in L09.

## L06: TypeScript evidence and pilot rule

**Outcome:** a real TypeScript adapter supplies coherent evidence and a narrow R1 comment-candidate path. Owners: `analysis/typescript`, its bridge where needed, `rules`, `engine`; sources: ADR-002/003 and the recorded native/coherence experiments.

**Bounded tasks:** L06.1 provisions verified tooling and integrates the required structural/project capabilities. L06.2 proves source/span binding, cross-file context, unsupported capabilities and native failures. L06.3 implements deterministic R1 candidate selection with evidence/counterevidence, retaining pending assessment when semantic judgment is needed.

**Exit:** real native fixtures distinguish narration, rationale/invariants and ambiguous comments; unresolved context stays visible. Oxc alone cannot stand for a complete project type graph. Add structured capabilities when a rule needs them; do not fabricate facts or precompute an unused universal graph.

**Measurement:** collect cold-load, repeated-query and whole-process-tree memory observations. Address major integration cost before broad rule expansion. Candidate selection and provider replay do not establish rule precision.

## L07: Replaceable judge and first complete analysis

**Outcome:** one explicitly selected TypeScript change traverses capture, evidence, assessment, policy, durable records and a CLI report. Owners: `judges`, `engine`, `policy`; sources: ADR-004, PRD US-08/US-09 and OPEN-03.

**Bounded tasks:** L07.1 specifies provider identity, endpoint/credential channels, trusted repository authorization and permitted evidence transmission. L07.2 implements the neutral contract, whole-phase deadline, bounded concurrency/retries and response/reference validation using controlled transport. L07.3 integrates the intended initial judge plus a controlled alternative adapter, and exposes one-shot analysis through the real engine.

**Exit:** eligible candidates are assessed automatically only after configuration/authorization; explicit no-model mode reports missing judgments. Both adapters preserve rule meaning and record/report contracts. Cover unsupported capabilities, malformed output, unknown references, timeout, cancellation and ambiguous retries. Admission and durable input/attempt capture precede transmission; secrets never enter retained exchanges.

**Boundary:** live-provider checks require configured repository authorization and a selected provider; never request secrets in chat. Freeze evaluation separation before renderer tuning. Pilot results remain advisory until their exact profile passes L13. This is the first analysis path, not v1 or a completed feedback/export workflow.

## L08: Feedback, exports and dataset operations

**Outcome:** early analyses become independently reconstructible evaluation/training material. Owner: `records` with thin application/CLI entrypoints; sources: ADR-005 and PRD US-05/US-06/US-07/US-09.

**Bounded tasks:** L08.1 implements attributed feedback, revisions and missed-issue reports. L08.2 completes rejected/no-candidate records and controlled non-candidate sampling with selection provenance, then exports versioned JSONL and required evidence. L08.3 implements explicit aggregation, lineage/deduplication and split preservation. L08.4 implements explicit purge with visible retained-reference and reclaimed-space effects, separating disposable caches from authoritative retention.

**Exit:** copying an export away from its workspace/database leaves every required reference resolvable. Failed export cannot appear complete. Changed source occurrences do not inherit labels silently; agent feedback, adjudication, suppression, deferral and false positives remain distinct. Duplicate inputs cannot inflate independent counts or erase feedback history. Export success alone deletes nothing.

**Milestone:** the pilot analysis/report/feedback/export path is available for internal use. The PRD's internal qualification slice also requires a justified thin-adapter counterexample; exercise that architectural case when R6 is available in L12.B. Subsequent languages and rules reuse this dataset path. Neither milestone reduces v1 coverage.

## L09: Client-owned resident session

**Outcome:** a client reuses analyzers and requests current reports through the same engine as the one-shot CLI. Owners: `session`, `engine`, shared supervision; sources: ADR-001/002/007 and PRD US-04/OPEN-04.

**Bounded tasks:** L09.1 freezes minimal transport/versioning, progress, cancellation, framing and backpressure. L09.2 implements bounded scheduling, configuration/source invalidation and stale-result rejection. L09.3 proves graceful close, EOF, abrupt client termination and owned-descendant cleanup on supported Linux/WSL2 environments.

**Exit:** every analysis still supplies its scope; repeated requests do not multiply workers without accounting. Older results cannot replace current reports. Stalled readers/repeated edits cannot cause unbounded queues. Configuration changes create a new generation without mutating in-flight history. CLI and resident observations agree for identical inputs.

**Measurement:** exercise B1 and warm reuse now, including store/projection/serialization and owned children. This is the reusable core boundary; Pi hooks/package and a global daemon remain outside v1.

## L10: PHP integration

**Outcome:** PHP participates in the same analysis, report, feedback, export and resident paths. Owner: `analysis/php` and its bridge; sources: ADR-002 and the Mago qualification evidence.

**Bounded tasks:** L10.1 verifies the provisioned Mago/SDK contract. L10.2 maps facts, spans, dependencies and partial framework/dynamic resolution into the existing evidence boundary. L10.3 integrates R1, resident invalidation and data/report conformance with real PHP fixtures.

**Exit:** mapping gaps/dynamic calls never become fabricated exact facts or proof of absence. Legitimate comments remain counterexamples. Native crash/reload/cancellation and changed callers preserve accurate generation/coverage. Rule, policy and dataset code is reused rather than copied into the adapter.

**Measurement:** check cold/warm/memory behavior on fixtures, then the declared PHP/TypeScript workload. Do not assume watch mode is faster or silently narrow quickcoin-platform's meaningful framework scope.

## L11: Rust integration

**Outcome:** Rust uses a replaceable rust-analyzer adapter within the same product boundaries. Owner: `analysis/rust` and shared supervision; sources: ADR-002/007 and the Rust qualification evidence.

**Bounded tasks:** L11.1 verifies the protocol and explicit build-script/procedural-macro execution policy. L11.2 implements required references/types and trustworthy original-source mappings, with explicit unsupported expansion cases. L11.3 integrates R1, one-shot/resident parity, cancellation/reload and durable records.

**Exit:** disabled research capabilities are not advertised as general support. Approximate mappings and unresolved calls remain coverage limits. Multi-crate/dependency changes invalidate affected facts; client exit leaves no owned analyzer descendants. Replacement does not require changing rules or records.

**Measurement:** measure startup, active and idle whole-tree memory on controlled workspaces and frozen Zerobox scenarios. Provisional rust-analyzer acceptance does not waive Rust budgets. Investigate overruns here, not only at release.

## L12: Full rule and language coverage

**Outcome:** all 27 R1–R9/language combinations have meaningful candidate, evidence/counterevidence and assessment paths within declared capabilities. Owners: `rules`, necessary native capabilities and grouping; sources: ADR-003 and PRD US-02/Evaluation Strategy.

**Bounded task unit:** one rule in one language, plus a separate integration task when shared behavior changes. Use identifiers such as `L12-R6-php`. Start from the real adapter and existing engine/store/judge boundaries. Never assign the whole coverage matrix as one implementation task.

| Workstream | Work | Required challenge cases |
| --- | --- | --- |
| L12.A | Close R1 parity and implement R2–R3. | Necessary comments; similar syntax with independent responsibilities; appropriate primitives/transport values. |
| L12.B | Implement R4–R6. | Current replaceability requirements; useful deep interfaces; thin adapters preserving security, ownership or dependency isolation. |
| L12.C | Implement R7–R9 and required bounded history evidence. | Necessary public contracts; explicit lifecycles; co-change/fan-out without shared conceptual responsibility; missing history. |

**Exit per cell:** positive, legitimate-negative and ambiguous cases traverse real native extraction and the public engine. Candidate misses and sampled non-candidates remain visible to evaluation. Counts, one implementation or absent capability cannot establish the verdict. Controlled provider responses validate orchestration, not precision. Cross-file attribution respects path overrides; grouping does not inflate actionable counts.

**Lot exit:** every cell has successful evidence/assessment examples and declared limits. Always returning unavailable does not count as delivery. Maintain a coverage/evidence ledger and freeze each holdout before tuning. Share facts where appropriate without merging different rule meanings. Extend native contracts only for demonstrated needs.

## L13: Qualification and selective CI blocking

**Outcome:** at least one exact rule/language/profile qualifies under the PRD's precision and other approved gates, and CI enforces it when explicitly enabled. Owners: `policy`, evaluation operations, CLI outcomes; sources: ADR-003/004/005 and PRD US-03/KPI-01/KPI-05.

**Bounded tasks:** L13.1 implements reproducible evaluation/replay with independent labels, sampling/lineage metadata and per-scope metrics. L13.2 evaluates frozen profiles, including precision, candidate/end-to-end recall, abstention and uncertainty. L13.3 implements qualification matching, prospective non-blocking comparison, activation/rollback and explicit CI artifact/outcome handling.

**Exit:** one combination passes the accepted precision target and OPEN-02's approved sample/confidence/recall/abstention protocol. Publish counts/denominators; zero positives or tiny perfect samples cannot qualify. Model scores, replay, another language's results or CI success do not substitute for adjudication. Activation remains explicit per project.

**Policy checks:** unqualified, stale or mismatched profiles cannot block as qualified violations. Missing required work fails visibly. Relevant model/renderer/evidence/settings changes require requalification. Preserve evidence/artifacts where possible, while recording/export failures stay explicit. No severity switch bypasses qualification.

Preparation/profile experiments can begin earlier on stable cells with independent data. If nothing qualifies, v1 remains unqualified; do not lower its release requirement silently.

## L14: Integrated performance and failure qualification

**Outcome:** the complete candidate meets resource and failure gates on declared workloads. Owners span engine, records, adapters and session; sources: ADR-007 and PRD KPI-02 through KPI-05.

**Bounded tasks:** L14.1 freezes protocol, host/load/cache conditions, repository captures and coverage for Pi-lens, quickcoin-platform and Zerobox. L14.2 measures B1–B4, actual model-phase timing and recording cost with raw observations/uncertainty. L14.3 validates mixed sessions, repeated edits, dependency/config changes, new/deleted callers, cancellation, slow readers, idle retention, storage failure and recovery.

**Exit:** include source capture, evidence work, durable writes, encoders and all owned children. Replay removes remote wait only for controlled local-path measurement; report actual end-to-end observations separately. Exports reconstruct independently. Cropped/incomplete workloads cannot satisfy resource or truthfulness gates.

**Cadence:** measurement started in L03–L11; this is integrated qualification, not the first performance investigation. Fix failures in their owner and repeat affected scenarios. Do not rerun the full corpus for routine edits. If a target cannot be met, obtain an explicit product decision instead of relabeling the workload or suppressing required rules.

## L15: Release candidate and distribution

**Outcome:** a new user can provision declared native requirements, configure the tool and run accepted workflows on supported Linux/WSL2 targets. Owners: packaging, release documentation, schema and entrypoint integration.

**Bounded tasks:** L15.1 finalizes supported architectures/packages and produces a reproducible artifact with prerequisites/integrity information. L15.2 prepares the version-pinned public schema, real `.anti-slop.jsonc` examples, credentials/authorization setup and local/CI instructions. L15.3 verifies installation/workflows from a clean location and assembles release acceptance evidence.

**Exit:** all upstream gates pass; examples/schema/binary support the same contract; the actual `$schema` URL resolves once publication is authorized. Ordinary analysis does not install/update analyzers. Document capabilities, failures, data locations, quota/purge/export behavior and qualification scope. Bundle no secrets or private corpus content.

**Publication boundary:** prepare reviewable artifacts before a requested publication. A local candidate is not a published release; remote schema/artifact publication needs an authorized destination. Hosting remains unspecified until selected. Do not claim first usable release completion while its public schema, first qualified blocking combination or resource gates remain outstanding.

## Open decisions and acceptance ownership

This mapping assigns the existing PRD gates without creating another specification.

| PRD gate | Resolution and verification lot |
| --- | --- |
| OPEN-01 | L01 entrypoint/selection/outcomes; L05 reports; L09 resident transport. |
| OPEN-02 | L01 protocol/separation; L12 profile evidence; L13 statistics and first passing combination. |
| OPEN-03 | L07 provider/credentials/transmission; native execution controls before relevant work in L06/L10/L11. |
| OPEN-04 | L03 captured generations, each adapter's actual view, L09 lifetime/cache mechanics, L14 resource proof. |
| OPEN-05 | L04 schema/journal/admission/recovery; L08 feedback/export/aggregation/purge. |
| OPEN-06 | L05 projection and TOON profile/codec. |
| OPEN-07 | Outside v1: actual future trained model, Pi triggers, additional platforms. |
| OPEN-08 | L02 configuration/schema; L12 cross-file attribution; L15 public schema availability. |

| Product acceptance | Delivery and verification |
| --- | --- |
| US-01 | L02–L03; adapter binding in L06/L10/L11. |
| US-02 | L05, L07 and L12. |
| US-03 | L13; clean-install CI checks in L15. |
| US-04 | L09 and L14. |
| US-05 | L04/L07/L08; integrated measurement in L14. |
| US-06 | L08, including revisions and missed-issue capture. |
| US-07 | L08 and L13; independent artifact reconstruction in L14/L15. |
| US-08 | L07 controlled replacement; L13 requalification/rollback. |
| US-09 | L02/L04/L07/L08; failure qualification in L14. |
| KPI-01 | L13, using the PRD threshold and approved evaluation protocol. |
| KPI-02, KPI-03 | Early measurements by implementing lot; complete ADR-007 boundaries in L14. |
| KPI-04 | L08 isolated reconstruction; integrated verification in L14. |
| KPI-05 | Per-lot failure/policy checks; combined acceptance in L13–L15. |

## Working on one task

Select one subtask, not the whole roadmap. Read `AGENTS.md`, its owners and the relevant repository skill; inspect current code and preserve unrelated changes. Translate the outcome into one minimal test through the actual public boundary, verify RED, implement GREEN and refactor. Documentation-only contracts use consistency/example validation rather than manufactured tests. Add dependencies only when needed, through the established workflow.

Use the cheapest relevant executable check during work. Current CLI coverage is `cargo test --frozen --test cli`; add/select focused targets as real features arrive instead of assuming future test commands exist. Format/diagnose changed code after behavior stabilizes. Broader checks require shared-contract impact or release qualification; state why. Record actual results and uncertainty, not implementation claims based on one passing fixture.

At handoff, record lot/subtask, acceptance evidence, affected interfaces, working-tree/commit state, unresolved gates and one bounded next task. Tie coverage and qualification to real evidence. Completing a narrow task never reduces v1 scope.

## First assignment

Start with **L01.1**, a bounded contract task. Suggested prompt for another agent:

> Read AGENTS.md, docs/PRD.md, the relevant accepted ADRs and L01 in docs/IMPLEMENTATION_ROADMAP.md. Complete only L01.1: specify the first CLI/request operations, explicit local-change and Git-base selection semantics, invalid-input behavior, and policy versus operational outcomes needed by the initial TypeScript analysis path. Inspect the existing CLI/tests and preserve current edits. Use docs/templates/task.md to bound the work. Include concrete valid, missing-scope, partial-work and failed-recording examples. Ask about unresolved product trade-offs; do not invent permanent semantics. Keep later resident transport, provider, storage and rule details in their assigned lots. Produce a reviewable English contract and a handoff for the first focused implementation test. Do not implement features, install dependencies, commit or publish as part of this contract-only task.

After the initial L01 contracts are settled, **L02 is the first production implementation lot**. O2, Pi and future training details need not be resolved before it begins.

## Outside the first release

O2 configuration composition, Pi package/hooks, macOS/native Windows, Trellis, a hosted collector/dashboard, automatic code rewriting and training/deploying a future replacement model remain later work under the PRD/ADRs. Preserve the required replacement/evolution boundaries without implementing those products early.

</details>
