# Anti-Slop

A Rust tool for evidence-based code review, designed for explicit CLI/CI analysis and a client-owned resident session. Language analyzers and model providers are replaceable boundaries. Retained evidence, execution records and attributed feedback will support evaluation and training exports.

## Current status

This repository contains [accepted architecture decisions](docs/decisions/README.md), a [PRD with approved product workflows and acceptance criteria](docs/PRD.md), qualification experiments and the first local Rust configuration slice. The [initial request, provenance and evaluation contracts](docs/PRD.md) are documented through AS-M001; its [mission state](.missions/AS-M001/state.json) owns independent acceptance. The executable provides help/version and read-only `config explain` with validated JSONC, advisory defaults, ordered overrides and winning origins. Its TOON-default and explicit JSON views share the Rust resolver/output owners. Analysis, resident sessions, rules, model calls, storage and dataset export are **not implemented**. Unknown commands fail; no command claims that a repository passed analysis.

The first usable version targets Linux, including WSL2. macOS and native Windows are later work. This is the selected support scope, not a completed platform qualification.

The initial dependency group is installed in the root Cargo package. Later engine, storage, codec and language dependencies remain in the [bootstrap design](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md) until the corresponding implementation begins. The existing research probes retain their own manifests and lockfiles; they are not workspace members.

Project configuration is specified as a root `.anti-slop.jsonc` with a versioned `$schema` link and path overrides in [ADR-008](docs/decisions/008-project-configuration-and-schema.md). That decision documents O1's limits and the compatible path toward O2 subproject configurations. The local loader and inspection boundary are available; see [the local configuration contract](docs/configuration.md). The generated local schema and headless VS Code editor-service conformance are available; the reserved public schema URL is unpublished; the installed TOML parser belongs to the earlier scaffold and does not define the selected format.

## Toolchain and dependencies

- One Cargo package: `anti-slop`; library: `anti_slop`; binary: `anti-slop`.
- Rust edition 2024, minimum Rust 1.96, development toolchain pinned to 1.96.1 with rustfmt and Clippy.
- Exact direct dependency requirements and `Cargo.lock` define the resolved dependency set. Keep both under version control.
- Development builds retain incremental compilation with `debug = 1`. Keep caches between focused TDD cycles.

The initial package setup uses Cargo through Socket Firewall (`sfw`). Recheck the exact registry version before adding or updating a dependency, and review the resolved graph before building it. New package downloads pass through Socket Firewall; existing cached archives are not newly scanned by that network control. Preserve caches rather than clearing them automatically. Keep lockfile changes with the corresponding manifest change.

The initial resolution applies a seven-day release-age window to registry packages, including transitives. Four transitive candidates were too recent: the lockfile instead selects `cfg-if 1.0.4`, `rustix 1.1.4`, `syn 3.0.5` and `unicode-ident 1.0.24`. Cargo does not enforce that window through this scaffold's configuration; repeat the registry check when deliberately changing the lockfile.

On the verified development host, Rust 1.96.1 and its required components were already installed. A fresh host must provision this toolchain and Socket Firewall separately before running the setup commands. Node is needed for independent TOON interoperability checks; no Node service is needed. PHP, Mago and rust-analyzer are not needed for this slice. Reference tooling has exact versions and a paired npm lockfile in `tools/config-conformance`. Its downloads also use `sfw` and the seven-day age check.

```sh
sfw cargo fetch --locked
cargo build --frozen
cargo run --frozen -- --help
cargo run --frozen -- --version
```

`--frozen` keeps routine builds offline and prevents lockfile changes. After an intentional dependency change, fetch through `sfw` before returning to the offline development loop.

## Development checks

After fetching locked dependencies through Socket Firewall, run the same
repository-wide verification used by CI:

```sh
(cd tools/config-conformance && sfw npm ci --ignore-scripts --no-audit --no-fund)
sfw cargo fetch --locked
bash scripts/ci/check.sh
```

The command checks formatting, CI policy regressions, changed-file whitespace,
the workspace build, all workspace test targets and doctests, Clippy, schema
regeneration and actual editor-service conformance. New Rust workspace members
and tests are discovered automatically; research probes are separate packages.
New production ecosystems need their own verification when introduced.

Before deliberately changing or fetching a new dependency graph, run
`python3 scripts/ci/policy.py dependencies --base <comparison-commit>`.
This checks added/changed crates.io and public npm lock entries against official
release metadata for seven-day age, yank status and matching locked integrity.
Omit `--base` to check the complete current graph. An unavailable comparison
commit or missing comparison lock also requires the complete candidate graph.
Unavailable or invalid metadata fails qualification; it never permits downloads.

Use a focused RED → GREEN → REFACTOR cycle for application behavior. Configuration and scaffold changes use proportionate executable validation. Expand test coverage with the implemented public boundary; avoid test copies of production logic and empty success paths.

## Continuous integration

The single `Linux CI` job runs on Ubuntu 24.04 for PRs targeting `main`, including
drafts, direct pushes to `main`, and manual dispatch. Other branch pushes and
schedules do not start it. A newer run cancels its superseded PR/branch run.
Actions use immutable commit pins; Rust 1.96.1, Node 24.19.0 and Socket Firewall
1.15.4 are pinned. Qualified package archives and Rust compilation outputs are
cached. Package fetches retain explicit `sfw` wrappers; subsequent Rust checks
are frozen. The job uses read-only repository, PR and Actions API permissions.

PR checks validate GitHub's test merge tree and retain a small JSON verification
receipt for 90 days. A push to `main` first checks for a merged PR whose final
head and successful CI receipt match the exact pushed Git tree. On a match, the
job links that successful run and skips setup, downloads, builds and tests.
The post-merge lookup is lightweight; there is no duplicate full verification.
Missing, expired, malformed, mismatched, failed or unreadable proof requires full
verification. Direct pushes also run fully; commit messages never determine
reuse. Manual dispatch always forces a full run. A changed base tree needs new
PR verification because it represents different combined inputs.

After the first successful Actions run, require `Linux CI` from GitHub Actions
on `main` with **Require branches to be up to date before merging**. Preserve
existing unrelated protection rules and leave administrator bypass enabled for
the approved owner override. That bypass permits direct pushes and can also
override PR checks; ordinary collaborators must satisfy the required check.
The workflow alone does not configure branch protection. CI is automated
repository verification, separate from mission acceptance or production
analysis qualification.

## Repository layout

| Location | Responsibility |
| --- | --- |
| `src/main.rs`, `src/cli.rs` | Thin read-only configuration CLI plus help/version. |
| `src/lib.rs`, `src/config/` | Shared resolution, validated inputs, immutable provenance and schema generation. |
| `src/workspace/`, `src/analysis/` | Reserved source-generation and language-analysis modules. |
| `src/rules/`, `src/judges/`, `src/policy/` | Reserved evidence rules, provider adapters and qualification decisions. |
| `src/records/`, `src/engine/` | Reserved durable records and shared orchestration. |
| `src/output.rs` | Shared configuration inspection projection and TOON/JSON encoding. |
| `src/output/`, `src/session/` | Reserved analysis report rendering and resident protocol. |
| `bridges/typescript/`, `bridges/php/` | Reserved native API workers; no packages installed here. |
| `tests/` | Executable CLI checks and reserved contract/fixture locations. |
| `benchmarks/` | Reserved workload scenarios; no performance runner yet. |
| `schemas/`, `examples/config_schema.rs` | Locally generated schema and deterministic generation/check tool. |
| `tools/config-conformance/` | Independent TOON decoder and actual local editor-service checks. |
| `docs/` | Product decisions, architecture and retained qualification evidence. |

Empty reserved directories use `.gitkeep` so the agreed layout survives a checkout. Replace each marker when the directory gets real content; do not introduce placeholder Rust interfaces merely to fill it.

Future workspace state belongs in an excluded data directory, defaulting to `.anti-slop/` under the approved design. This scaffold does not create it. Real source snapshots, datasets, credentials and local corpus paths must not become committed test fixtures automatically.

## Working with agents

Start an implementation or documentation task with [AGENTS.md](AGENTS.md), this current-status section and the relevant [accepted ADRs](docs/decisions/README.md). Read the [PRD](docs/PRD.md) for product workflows and release acceptance, observing its status and open specifications. The [bounded task template](docs/templates/task.md) remains available for standalone work outside a mission.

The [progressive first-release roadmap](docs/IMPLEMENTATION_ROADMAP.md) preserves the original L00–L15 plan. AS-M001 is completed with independent acceptance in its [state](.missions/AS-M001/state.json). The AS-M002 configuration slice was recovered from the original implementation and validator chat records after its uncommitted worktree disappeared. The [recovery record](.missions/AS-M002/recovery/README.md) distinguishes historical acceptance, fresh verification and missing mission records. This branch is a review candidate; the original complete state/history and packet evidence have not been recovered. AS-M003 remains unstarted.

The [repository skill catalog](.agents/skills/README.md) contains workflows for rule evidence and analyzer qualification. An agent can read these files directly; automatic skill discovery depends on its harness and is not assumed. The [workflow validation notes](docs/agent-workflow-validation.md) state the checks performed and their limits.

## Decisions and research

The [ADR index](docs/decisions/README.md) is the current source of architectural decisions and links product decisions to their owner in the [PRD](docs/PRD.md). Accepted decisions do not establish implemented behavior. Historical design documents below retain discussion and proposals; research retains the original measured inputs and limitations.

- [Historical design discussion](docs/brainstorming-decisions/2026-09-22-anti-slop-design.md)
- [Historical architecture proposal](docs/brainstorming-decisions/2026-09-22-anti-slop-architecture.md)
- [Approved bootstrap proposal and deferred dependency candidates](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md)
- [Language qualification](docs/research/2026-09-22-language-qualification/README.md)
- [Source-generation coherence experiment](docs/research/2026-09-22-source-coherence/README.md)

All product interfaces, source code and project documentation are in English.
