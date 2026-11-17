# Anti-Slop

A Rust tool for evidence-based code review, designed for explicit CLI/CI analysis and a client-owned resident session. Language analyzers and model providers are replaceable boundaries. Retained evidence, execution records and attributed feedback will support evaluation and training exports.

## Current status

This repository contains [accepted architecture decisions](docs/decisions/README.md), a [PRD with approved product workflows and acceptance criteria](docs/PRD.md), qualification experiments and the first local Rust configuration slice. The [initial request, provenance and evaluation contracts](docs/PRD.md) are documented through AS-M001; its [mission state](.missions/AS-M001/state.json) owns independent acceptance. The shared library provides bounded four-selector source capture through `workspace::capture`, independently accepted and merged through [the AS-M003 delivery](https://github.com/abdwhb-png/anti-slop/pull/2). Its [mission state](.missions/AS-M003/state.json) records completed local closure after the [reviewed historical-reference recovery](.missions/AS-M003/evidence/recovery/README.md); the record correction is merged through [PR #3](https://github.com/abdwhb-png/anti-slop/pull/3) into the starting baseline. The shared library also exposes durable capture records through `records::Store`; the [AS-M004 mission state](.missions/AS-M004/state.json) owns their current qualification and acceptance. The executable provides help/version and read-only `config explain` with validated JSONC, advisory defaults, ordered overrides and winning origins. Its TOON-default and explicit JSON views share the Rust resolver/output owners. Analysis, resident sessions, rules, model calls, reports and dataset export are **not implemented**; durable capture recording is a library capability, not a CLI command. Unknown commands fail; no command claims that a repository passed analysis.

The first usable version targets Linux, including WSL2. macOS and native Windows are later work. This is the selected support scope, not a completed platform qualification.

The initial dependency group and the SQLite storage dependency are installed in the root Cargo package. Later engine, codec and language dependencies remain in the [bootstrap design](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md) until the corresponding implementation begins. The existing research probes retain their own manifests and lockfiles; they are not workspace members.

Project configuration is specified as a root `.anti-slop.jsonc` with a versioned `$schema` link and path overrides in [ADR-008](docs/decisions/008-project-configuration-and-schema.md). That decision documents O1's limits and the compatible path toward O2 subproject configurations. The local loader and inspection boundary are available; see [the local configuration contract](docs/configuration.md). The generated local schema and headless VS Code editor-service conformance are available; the reserved public schema URL is unpublished; the installed TOML parser belongs to the earlier scaffold and does not define the selected format.

## Source capture through the library

Call `workspace::capture` with exactly one explicit `Selection`. `CaptureRequest::new` requires it; the Rust type has no default, missing or conflicting selector state. The existing executable remains a configuration inspector; there is no analysis/capture CLI command.

| Selection | Before and selected after inputs |
| --- | --- |
| `Staged` | Once-resolved HEAD to index; after-side role `Index`. Worktree-only and untracked changes are absent. |
| `Unstaged` | Index to actual working files, including nonignored untracked inputs; HEAD is not required. Equal index/working bytes and modes produce no change. |
| `Local` | Once-resolved HEAD directly to working files, including nonignored untracked inputs. An index edit canceled by working bytes produces no local change. |
| `Base(reference)` | The supplied reference, resolved once, directly to working files; never merge-base or an implicit branch fallback. |

```rust,no_run
use anti_slop::workspace::{capture, CaptureRequest, Selection};
use std::path::Path;

let request = CaptureRequest {
    context: vec!["src/consumer.ts".into()],
    exclusions: vec!["private.txt".into()],
    ..CaptureRequest::new(Selection::Base("release".into()))
};
let captured = capture(Path::new("/path/to/repository"), &request)?;
let policy = captured.configuration().resolve(Path::new("src/consumer.ts"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The read-only result retains both available comparison sides, actual bytes, paths, roles, regular/executable modes, available Git blob identities and comparison/configuration generations. A deletion has an absent after-side; an empty file has zero bytes; a valid empty selection has no changes and still binds its observed inputs. Working and context roles remain separate at the same path, sharing one captured working-file observation. Earlier sources and compiled configuration remain inspectable after the originals change or disappear. Byte ranges bind exact retained source path, role and generation; offsets are bytes, including empty/EOF and partial Unicode slices.

Rename attribution is deterministic: pair a deletion and addition only when their exact bytes and normalized mode agree and the content group contains exactly one of each. Differing `before.path` and `after.path` identify that attribution; `Change.path` is the after path (or the old path for a deletion). The rule is exact-content evidence, not proof of the author's physical move operation. Indistinguishable groups remain explicit unpaired deletions/additions. Both names are checked for exclusions before acquiring their content.

One configuration is captured and compiled per attempt; every path view uses its retained bytes, policies and origins. Existing explicit external configuration selection stays supported. Caller exclusions are normalized literal workspace-relative paths, independent of override globs. Overlap with any required comparison candidate, explicit context or selected configuration returns `CaptureError::Excluded` with observed paths/roles and no partial capture. A candidate cannot be read through an exclusion merely to decide that it was unchanged. Names must be UTF-8; spaces, tabs, newlines and literal backslashes are preserved. Source symlinks, submodules, unmerged indexes and unavailable required inputs fail explicitly. Working selectors also reject indexes with assume-unchanged or skip-worktree entries; intent-to-add is not staged content and is a working addition.

Git supplies only object/index/tree/path inventories and raw stored blobs. Working comparisons enumerate the baseline tree or logical index, current index paths and nonignored untracked paths, then Rust compares actual bytes and modes. This intentionally inspects unchanged comparison candidates too: raw Git working diffs can run clean/process filters while checking racy timestamps. No project helper/filter, external diff, model or fetch is allowed. No normalized/filter-transformed working view is substituted. Descriptor-anchored Linux reads never follow source symlinks, including intermediate components; configuration symlinks retain their separately supported selection semantics.

Acquisition limits are 512 input observations/roles, 1 MiB per content input and 16 MiB total admitted content per attempt. Unchanged comparison candidates, absence observations and repeated context roles consume admission; `acquired_files`/`acquired_bytes` report that work separately from actual retained inputs. Thus a small edit in a large baseline can fail the bound explicitly. Caller lists have at most 512 entries each and normalized paths/configuration arguments at most 4,096 bytes. Each metadata inventory is capped at 2 MiB; working capture holds baseline/index-path/untracked inventories with at most one extra bounded inventory during verification. Each owned Git command has a two-second deadline and 8 KiB stderr cap, with one Git command at a time. Blob sizes are checked before body requests; regular-file reads enforce caps while growing. Verification uses at most one extra 1 MiB content input at a time. These are admission/content/buffer bounds, not a total allocator, compiled-configuration or owned-process RSS quota.

HEAD/base is pinned once. Captured object identities supply stored bytes; relevant metadata inventories and exact working/configuration bytes are observed twice, with one retry before explicit instability. Capture identity includes those inventories and mutable observations even when the selected set is empty. Git leaders remain unreaped through group signaling, then are reaped before reader threads join, including timeout/held-descendant-pipe cleanup. The achieved scope is bounded relevant observations and owned retained inputs; global filesystem atomicity, detection of edits that revert between observations, perpetual freshness, arbitrary dependency hermeticity and native-overlay coherence are not claimed. Capture performs no analysis, model call, storage or export.

For the complete small nonvisual journey, run `cargo test --frozen --test workspace four_comparisons_select_their_exact_sides_and_independent_context -- --exact --nocapture`; the real fixture has distinct base/merge-base/HEAD/index/working bytes, staged-only and ignored/untracked inputs, plus independent context. Failure, mutation and retained-range cases are in the same public-boundary test target. Run `python3 .missions/AS-M003/evidence/P02/measure.py staged --output .missions/AS-M003/evidence/P02/trial-staged-001` (or `unstaged`, `local`, `base`, with a fresh output directory) for one-file and five-file/200-changed-line timing, acquisition/retention counts and owned-process sampling. Existing output paths are refused before build/sampling; choose a new directory for each relaunch. [P02 evidence](.missions/AS-M003/evidence/P02/) retains its observations, limits and the explicitly approved TDD sequencing exception; [P03 evidence](.missions/AS-M003/evidence/P03/) retains renewed measurements after the mode correction. Historical evidence stays unchanged. This is a local library demonstration with no hosted trial; the implementation was published and merged through [the AS-M003 delivery](https://github.com/abdwhb-png/anti-slop/pull/2).

## Durable capture records

`records` stores a completed `workspace::Capture` and reads its exact bytes and provenance back later, including after the originating files and Git objects are gone. The store is a local SQLite database; the interface is library-only.

```rust,no_run
use anti_slop::{
    records::{Budget, Store},
    workspace::{CaptureRequest, Selection, capture},
};
use std::path::Path;

let captured = capture(
    Path::new("/path/to/repository"),
    &CaptureRequest::new(Selection::Local),
)?;
let mut store = Store::open_with_budget(
    Path::new("/workspace/.anti-slop/records.sqlite3"),
    Budget::default(), // 10 GiB per workspace
)?;
let saved = store.save_capture(&captured)?;
let retained = store.read_capture(saved.id)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

A save retains the source bytes, roles, generations, Git blob and mode identities, comparison selection/commit, configuration identity and bytes in one SQLite transaction. A returned `SavedCapture` follows a completed commit and carries the measured retained footprint. Reads verify a records-owned integrity digest over the complete membership, provenance and bytes; missing, altered, duplicate or extra evidence returns an explicit corruption error. Retained reads include deletion-versus-empty distinctions, valid empty captures and independent occurrences of equal content.

The parent directory of the database path must already exist; the example assumes `/workspace/.anti-slop/` does.

WAL mode with `synchronous = FULL` provides the selected durability; SQLite removes the WAL when the last connection closes. The store enforces an application admission budget (10 GiB default, warning at 80%, explicit refusal at the limit). The reservation covers database pages, WAL/SHM/journal files, checkpoint duplication and the existing WAL backlog; it is an engineering allowance under the bundled SQLite settings, not a hard filesystem quota. Opening a store below its retained size does not prevent retained reads.

The store uses schema version 1, which has not shipped; no migration history is provided. Purge/export, analysis, report rendering and admission for model work are later outcomes. `cargo test --frozen --test records` exercises the public library boundary; the record code has no analysis or CLI command.

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
| `src/workspace/` | Four explicit source comparisons, retained generations and bounded read-only Git/filesystem acquisition. |
| `src/analysis/` | Reserved language-analysis module. |
| `src/rules/`, `src/judges/`, `src/policy/` | Reserved evidence rules, provider adapters and qualification decisions. |
| `src/records/` | Durable capture records, storage integrity and admission. |
| `src/engine/` | Reserved shared orchestration. |
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

The [progressive first-release roadmap](docs/IMPLEMENTATION_ROADMAP.md) preserves the original L00–L15 plan. AS-M001 is completed with independent acceptance in its [state](.missions/AS-M001/state.json). The AS-M002 configuration slice was recovered from the original implementation and validator chat records after its uncommitted worktree disappeared. Its [recovery record](.missions/AS-M002/recovery/README.md) distinguishes historical acceptance, fresh verification and missing mission records; AS-M003 recovery does not recreate those missing AS-M002 records. AS-M003 source capture is independently accepted and merged. Its [mission state](.missions/AS-M003/state.json) preserves the historical verdicts and records completed local closure after [reviewed reference recovery](.missions/AS-M003/evidence/recovery/README.md). Delivery of the record correction and later mission progression remain separate.

The [repository skill catalog](.agents/skills/README.md) contains workflows for rule evidence and analyzer qualification. An agent can read these files directly; automatic skill discovery depends on its harness and is not assumed. The [workflow validation notes](docs/agent-workflow-validation.md) state the checks performed and their limits.

## Decisions and research

The [ADR index](docs/decisions/README.md) is the current source of architectural decisions and links product decisions to their owner in the [PRD](docs/PRD.md). Accepted decisions do not establish implemented behavior. Historical design documents below retain discussion and proposals; research retains the original measured inputs and limitations.

- [Historical design discussion](docs/brainstorming-decisions/2026-09-22-anti-slop-design.md)
- [Historical architecture proposal](docs/brainstorming-decisions/2026-09-22-anti-slop-architecture.md)
- [Approved bootstrap proposal and deferred dependency candidates](docs/brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md)
- [Language qualification](docs/research/2026-09-22-language-qualification/README.md)
- [Source-generation coherence experiment](docs/research/2026-09-22-source-coherence/README.md)

All product interfaces, source code and project documentation are in English.
