# Anti-Slop

A Rust tool for evidence-based code review, designed for explicit CLI/CI analysis and a client-owned resident session. Language analyzers and model providers are replaceable boundaries. Retained evidence, execution records and attributed feedback will support evaluation and training exports.

## Current status

This repository contains [accepted architecture decisions](docs/decisions/README.md), a [PRD with approved product workflows and acceptance criteria](docs/PRD.md), qualification experiments and the initial Rust scaffold. The executable provides help and version information only. Analysis, resident sessions, configuration loading, rules, model calls, storage and dataset export are **not implemented**. Unknown commands fail; no command claims that a repository passed analysis.

The first usable version targets Linux, including WSL2. macOS and native Windows are later work. This is the selected support scope, not a completed platform qualification.

The initial dependency group is installed in the root Cargo package. Later engine, storage, codec and language dependencies remain in the [bootstrap design](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md) until the corresponding implementation begins. The existing research probes retain their own manifests and lockfiles; they are not workspace members.

Project configuration is specified as a root `.anti-slop.jsonc` with a versioned `$schema` link and path overrides in [ADR-008](docs/decisions/008-project-configuration-and-schema.md). That decision documents O1's limits and the compatible path toward O2 subproject configurations. The loader and published schema are not available yet; the installed TOML parser belongs to the earlier scaffold and does not define the selected format.

## Toolchain and dependencies

- One Cargo package: `anti-slop`; library: `anti_slop`; binary: `anti-slop`.
- Rust edition 2024, minimum Rust 1.96, development toolchain pinned to 1.96.1 with rustfmt and Clippy.
- Exact direct dependency requirements and `Cargo.lock` define the resolved dependency set. Keep both under version control.
- Development builds retain incremental compilation with `debug = 1`. Keep caches between focused TDD cycles.

The initial package setup uses Cargo through Socket Firewall (`sfw`). Recheck the exact registry version before adding or updating a dependency, and review the resolved graph before building it. New package downloads pass through Socket Firewall; existing cached archives are not newly scanned by that network control. Preserve caches rather than clearing them automatically. Keep lockfile changes with the corresponding manifest change.

The initial resolution applies a seven-day release-age window to registry packages, including transitives. Four transitive candidates were too recent: the lockfile instead selects `cfg-if 1.0.4`, `rustix 1.1.4`, `syn 3.0.5` and `unicode-ident 1.0.24`. Cargo does not enforce that window through this scaffold's configuration; repeat the registry check when deliberately changing the lockfile.

On the verified development host, Rust 1.96.1 and its required components were already installed. A fresh host must provision this toolchain and Socket Firewall separately before running the setup commands. No Node, PHP, Mago or rust-analyzer installation is needed to build this initial scaffold.

```sh
sfw cargo fetch --locked
cargo build --frozen
cargo run --frozen -- --help
cargo run --frozen -- --version
```

`--frozen` keeps routine builds offline and prevents lockfile changes. After an intentional dependency change, fetch through `sfw` before returning to the offline development loop.

## Development checks

```sh
cargo test --frozen --test cli
cargo fmt --all -- --check
cargo clippy --frozen --all-targets -- -D warnings
```

Use a focused RED → GREEN → REFACTOR cycle for application behavior. Configuration and scaffold changes use proportionate executable validation. Expand test coverage with the implemented public boundary; avoid test copies of production logic and empty success paths.

## Repository layout

| Location | Responsibility |
| --- | --- |
| `src/main.rs`, `src/cli.rs` | Minimal executable entrypoint and command-line metadata. |
| `src/lib.rs` | Library entrypoint; no application operations yet. |
| `src/workspace/`, `src/analysis/` | Reserved source-generation and language-analysis modules. |
| `src/rules/`, `src/judges/`, `src/policy/` | Reserved evidence rules, provider adapters and qualification decisions. |
| `src/records/`, `src/engine/` | Reserved durable records and shared orchestration. |
| `src/output/`, `src/session/` | Reserved report rendering and resident protocol. |
| `bridges/typescript/`, `bridges/php/` | Reserved native API workers; no packages installed here. |
| `tests/` | Executable CLI checks and reserved contract/fixture locations. |
| `benchmarks/` | Reserved workload scenarios; no performance runner yet. |
| `examples/` | Configuration examples will follow an implemented configuration contract. |
| `docs/` | Product decisions, architecture and retained qualification evidence. |

Empty reserved directories use `.gitkeep` so the agreed layout survives a checkout. Replace each marker when the directory gets real content; do not introduce placeholder Rust interfaces merely to fill it.

Future workspace state belongs in an excluded data directory, defaulting to `.anti-slop/` under the approved design. This scaffold does not create it. Real source snapshots, datasets, credentials and local corpus paths must not become committed test fixtures automatically.

## Working with agents

Start an implementation or documentation task with [AGENTS.md](AGENTS.md), this current-status section and the relevant [accepted ADRs](docs/decisions/README.md). Read the [PRD](docs/PRD.md) for product workflows and release acceptance, observing its status and open specifications. The [bounded task template](docs/templates/task.md) remains available for standalone work outside a mission.

The [progressive first-release roadmap](docs/IMPLEMENTATION_ROADMAP.md) preserves the original L00–L15 plan and presents W01 for approval. Only [AS-M001](docs/missions/AS-M001/r1.md) has a detailed Mission Contract; its [state](.missions/AS-M001/state.json) is `draft`. The roadmap links the supplied mission-driven-dev workflow. Mission preparation, execution, delegation and commits require their respective authority; historical prompts and draft records do not authorize launch. Completed behavior remains recorded in the current-status section above.

The [repository skill catalog](.agents/skills/README.md) contains workflows for rule evidence and analyzer qualification. An agent can read these files directly; automatic skill discovery depends on its harness and is not assumed. The [workflow validation notes](docs/agent-workflow-validation.md) state the checks performed and their limits.

## Decisions and research

The [ADR index](docs/decisions/README.md) is the current source of architectural decisions and links product decisions to their owner in the [PRD](docs/PRD.md). Accepted decisions do not establish implemented behavior. Historical design documents below retain discussion and proposals; research retains the original measured inputs and limitations.

- [Historical design discussion](docs/brainstorming-decisions/2026-09-22-anti-slop-design.md)
- [Historical architecture proposal](docs/brainstorming-decisions/2026-09-22-anti-slop-architecture.md)
- [Approved bootstrap proposal and deferred dependency candidates](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md)
- [Language qualification](docs/research/2026-09-22-language-qualification/README.md)
- [Source-generation coherence experiment](docs/research/2026-09-22-source-coherence/README.md)

All product interfaces, source code and project documentation are in English.
