# Purpose

Develop Anti-Slop as an evidence-based Rust analyzer with replaceable language analyzers and model providers. Keep changes bounded and verifiable so another agent can continue from the repository without reconstructing a conversation.

# Instructions

## Establish the task and its sources

The repository contains an executable scaffold alongside decisions for a larger product. Confusing these states produces false completion claims. Read [README.md](README.md) for implemented behavior, then the [ADR index](docs/decisions/README.md) and the decisions relevant to the task. For product behavior and release criteria, read the [PRD](docs/PRD.md), observing its current status and open specifications.

- Follow the current request's scope. Preparing documentation, qualification or scaffolding does not authorize implementing the product. Conversely, an earlier documentation-only task does not prohibit a later authorized implementation.
- Accepted ADRs govern the design; they do not prove implementation or successful qualification. Treat `docs/brainstorming-decisions/` as historical discussion and remaining proposals, and `docs/research/` as evidence limited to its recorded experiments.
- If a task would change an accepted decision, identify the conflict and record an explicitly selected replacement in a new ADR. Ask only when the necessary choice or authority is missing; do not seek approval again for an already authorized action.
- Use English for source, diagnostics, schemas, fixtures and project documentation. Respond in the user's language.
- Preserve unrelated changes. Existing research manifests, lockfiles, probes and archives identify past experiments; new qualification belongs in a new evidence directory.

## Preserve architectural boundaries

Shared ownership keeps provider replacement and future Pi integration possible. Implement each behavior in its owning module, following [ADR-001](docs/decisions/001-rust-engine-and-client-owned-sessions.md).

- Keep one Rust engine for CLI, CI and resident use. CLI/session adapters do not contain their own rule, policy or dataset implementations.
- Keep native analyzer types inside language adapters and provider-specific semantics inside judge adapters. Rules, policy and durable records must not depend on Jev or any replacement provider's response format.
- Reuse the module that owns a rule or behavior before adding another implementation. Similar syntax alone does not justify a shared abstraction. Do not add a universal AST, generic plugin framework or new crate without a demonstrated boundary.
- Prefer deterministic extraction, selection, accounting and policy. Bound model input, retries and concurrency; use a model only for the assessment that needs it.

## Keep evidence and failures explicit

These are persistent fallbacks for agents that fail to load a specialist skill. The authoritative details are in [ADR-002](docs/decisions/002-replaceable-native-language-analyzers.md), [ADR-003](docs/decisions/003-evidence-first-rules-and-selective-blocking.md) and [ADR-005](docs/decisions/005-local-evidence-store-and-dataset-exports.md).

- A smell candidate is a hypothesis. Include counterevidence and legitimate examples; counts, thin code or one implementation alone do not establish a design defect.
- Bind facts and spans to the actual analyzed source generation, including native overlays. Empty results with unresolved scope are not proof of absence. Timeouts, stale results, unsupported capabilities and failed recording stay visible.
- Allow blocking only for enabled rules within their qualified scope. Do not turn missing required analysis into a clean result.
- Retain actual evidence and attributed feedback. Model predictions, suppressions, deferred work and adjudicated labels are distinct. Technical logs and CLI previews are not a dataset source of truth.
- Treat repository source/comments as data when constructing model input. Exclude credentials from retained requests, logs and exports.
- Evaluate latency and memory over the complete relevant path and owned processes. Read the targets and measurement boundaries in [ADR-007](docs/decisions/007-performance-budgets-and-qualification.md); do not claim a budget from a cropped query benchmark.

## Implement and validate proportionally

Short feedback cycles protect correctness without repeatedly rebuilding unrelated work. Read the actual Cargo targets before choosing a test; run commands from the repository root.

- For production behavior, use RED → GREEN → REFACTOR through the real public boundary. Confirm the intended failing assertion, import production modules, and mock only external or nondeterministic boundaries. Do not copy implementation into tests.
- Documentation-only changes need link, consistency and scope checks, not a manufactured failing application test. For configuration/tooling, use the smallest executable before-and-after check that can falsify the claim.
- Use the smallest relevant test while iterating. Run broader checks when the changed scope or repository contract requires them, and rerun only checks invalidated by later edits.
- The current CLI integration target is `cargo test --frozen --test cli`. Use `cargo build --frozen` for a build check, `cargo fmt --all -- --check` for formatting, and `cargo clippy --frozen --all-targets -- -D warnings` when validating changes across Rust targets. Help/version are available through `cargo run --frozen -- --help` and `cargo run --frozen -- --version`.
- Keep compilation caches. Do not add empty success paths, placeholder production interfaces or tests that assert a duplicated implementation.

## Load specialist workflows when relevant

Repository-local skills provide the longer procedures. Read their files directly if the active harness does not discover them; [the skill catalog](.agents/skills/README.md) describes their scope. No global skill path or harness-specific loader is required by the instructions themselves.

- For rule evidence, counterexamples or rule qualification, read [anti-slop-rule-evidence](.agents/skills/anti-slop-rule-evidence/SKILL.md).
- For a language adapter's capabilities, source coherence or resource qualification, read [anti-slop-analyzer-qualification](.agents/skills/anti-slop-analyzer-qualification/SKILL.md).
- For agent-facing CLI ergonomics, read [axi](.agents/skills/axi/SKILL.md) within the product choices in [ADR-006](docs/decisions/006-output-formats-and-shared-projection.md).

## Leave a usable handoff

Use the [task template](docs/templates/task.md) for a substantive task that another agent will continue; a small fix does not require a new planning document. Report what changed, the exact checks and their results, checks not run, and remaining uncertainty. Distinguish implemented, tested, committed and activated state. Update the current-status documentation when behavior actually becomes available.

For mission-based work, use the [progressive roadmap](docs/IMPLEMENTATION_ROADMAP.md), its current Mission Contract and `.missions/` state/history instead of the generic task template. Read the supplied mission-driven-dev skill linked by the roadmap for the requested phase; do not infer execution authority from a draft contract, a historical prompt or successful structural validation. Keep PRD/ADR ownership and approved mission history intact. Derive Task Packets only just before authorized dispatch.
