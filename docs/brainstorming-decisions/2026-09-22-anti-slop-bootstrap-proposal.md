# Anti-Slop: repository and dependency bootstrap proposal

> Historical approved bootstrap scope. [ADR-001](../decisions/001-rust-engine-and-client-owned-sessions.md) now owns the package/module decision; the [root README](../../README.md#current-status) and Cargo files record actual installed state. Deferred package candidates and detailed interfaces below remain proposals, not instructions to install or implement them all.

Date: 2026-09-22

Status: **approved on 2026-09-22 for bootstrap A**, with the user's explicit constraint that application features must not be implemented yet. Bootstrap execution is recorded in the root [README](../../README.md). Groups B/C, bridge installations and substantive application behavior remain deferred.

## Destination and scope

Prepare a small, buildable Rust project in which the [agreed architecture](2026-09-22-anti-slop-architecture.md) can be implemented incrementally. Preserve [D1–D17 and R1–R9](2026-09-22-anti-slop-design.md), English product surfaces, replaceable judges and analyzers, and the shared CLI/resident engine. This proposal selects code locations and dependency candidates; it does not claim that the engine, rule coverage, or performance budgets are implemented.

At proposal time the checkout contained design documents and qualification probes, with no production Cargo package to reorganize. Keep those probes and their historical lockfiles unchanged: they identify previously measured experiments.

## BP1 — One Rust package, responsibility-based modules

Use package `anti-slop`, library `anti_slop`, and binary `anti-slop`, with edition **2024** and minimum Rust **1.96**, matching the qualified Oxc 0.150.0 MSRV. The proposal named Rust 1.96.0; bootstrap pins the already installed **1.96.1** toolchain with rustfmt and Clippy. The host's 1.96.0 compiler is registered under the moving `stable` channel, so using the installed numbered patch gives a reproducible pin without downloading another toolchain. This does not change the compiler identity recorded in earlier qualification experiments. Set `publish = false` until distribution is designed. One package permits direct library tests and keeps CLI and resident behavior together; extracting a crate later requires a concrete independent consumer or build boundary.

The following is the target map. Create source files when their responsibilities are implemented, rather than filling the tree with placeholder traits, `todo!()` functions, or successful empty analyses.

```text
anti-slop/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── .gitignore
├── src/
│   ├── lib.rs                    # Public application operations
│   ├── main.rs                   # Process entrypoint
│   ├── cli.rs                    # Arguments, command dispatch, exit status
│   ├── config.rs                 # Effective configuration and validation
│   ├── workspace/                # M1: scope, source generations, Git/history
│   ├── analysis/                 # M2: facts, capabilities, native sessions
│   │   ├── typescript/           # Oxc and native TypeScript mapping
│   │   ├── php/                  # Mago SDK mapping
│   │   ├── rust/                 # rust-analyzer mapping
│   │   ├── lsp.rs                # Shared native LSP client transport
│   │   └── process.rs            # Owned children, limits, shutdown
│   ├── rules/                    # M3: R1–R9, evidence and counterevidence
│   ├── judges/                   # M4: neutral assessment interface
│   │   └── jev.rs                # Initial provider rendering and transport
│   ├── policy/                   # M5: qualification and blocking decisions
│   ├── records/                  # M6: durable runs, evidence, feedback, export
│   │   └── migrations/           # Ordered SQL migrations owned by records
│   ├── engine/                   # M7: sequencing, budgets, cancellation
│   ├── output/                   # One report projection, TOON/JSON encoders
│   └── session/                  # Versioned JSON-RPC over stdio
├── bridges/
│   ├── typescript/               # Small Node helper for the typed TS API
│   │   ├── package.json
│   │   ├── bun.lock
│   │   ├── bunfig.toml
│   │   └── worker.mjs
│   └── php/                      # Small worker importing the Mago SDK
│       ├── composer.json
│       ├── composer.lock
│       └── worker.php
├── tests/
│   ├── cli.rs                    # Binary behavior through its public boundary
│   ├── contracts.rs              # Registers focused contract test modules
│   ├── contracts/               # Formats, store, sessions and adapters
│   ├── support/                 # Test setup; imports real production modules
│   └── fixtures/
│       ├── typescript/
│       ├── php/
│       ├── rust/
│       └── reports/
├── benchmarks/
│   ├── scenarios/               # Cold, warm, report, cancellation and export
│   └── corpus.example.toml       # Logical repo names; local paths configured separately
├── examples/
│   └── anti-slop.toml
└── docs/                         # Existing design and research retained
```

Small modules can begin as `workspace.rs`, for example, and become directories when private implementation files justify it. Top-level Rust modules need ordinary `mod.rs` or same-named module files; the map abbreviates those mechanical details.

Types live with the responsibility that owns their meaning. `rules` consumes normalized observations, `policy` consumes assessments, and `records` persists versioned records; none imports provider SDK types. `jev.rs` is the only place with Jev-specific request/response semantics. A new judge supplies another adapter and qualification profile without changing the dataset's canonical evidence format.

`records` owns both retention and JSONL/evidence-bundle export. Operational `tracing` logs are separate from durable analysis telemetry and training cases. This prevents debug logging from becoming a second, incomplete dataset writer. Schemas for foreign consumers are generated from the Rust boundary types once those types exist; generated files are checked for drift, not edited as another source of truth.

The resident session and native LSP clients share appropriate bounded I/O mechanics, but have distinct framing: newline-delimited JSON-RPC for Anti-Slop and `Content-Length` framing for native LSP. No HTTP server is needed for the first client-owned session.

## BP2 — Dependencies staged by the code that needs them

Exact candidates below were checked against live official registry metadata on 2026-09-22. Rust candidates are non-yanked and were published at least seven days earlier. [The metadata snapshot](../research/2026-09-22-bootstrap-dependencies.json) records source URLs, publication dates, declared MSRVs, checksums and selected features. It is research evidence, not a second dependency lockfile.

Metadata does not establish that the combined dependency graph builds or meets resource budgets. Resolve and inspect that graph after approval, keep exact direct requirements and commit `Cargo.lock`. Retain qualified analyzer versions even where newer releases exist.

### A — Install when bootstrapping the Rust project

| Dependency | Exact candidate | Role and feature selection |
| --- | --- | --- |
| `clap` | `4.6.7` | CLI; disable defaults, enable `std`, `derive`, `help`, `usage`, `error-context`. |
| `serde` | `1.0.229` | Shared typed serialization; enable `derive`. |
| `serde_json` | `1.0.151` | JSON reports, session envelopes and JSONL. |
| `toml` | `1.1.6+spec-1.1.0` | Configuration; disable defaults, enable `std`, `serde`, `parse`. Cargo requirements use `=1.1.6`; build metadata does not participate in version matching. |
| `thiserror` | `2.0.20` | Typed application errors; preserve explicit failure categories. |
| `tracing` | `0.1.44` | Attributed technical events. |
| `tracing-subscriber` | `0.3.23` | Stderr diagnostics; disable defaults, enable `fmt`, `env-filter`. |
| `tempfile` | `3.27.0` | Development dependency for isolated filesystem/database tests. |
| `assert_cmd` | `2.2.2` | Development dependency for CLI tests. Use ordinary structured assertions where sufficient. |

This installs seven production dependencies and two direct test dependencies. It does not start analyzers or initialize a user's record store.

### B — Add with the first engine, recording and report implementation

| Dependency | Exact candidate | Role and feature selection |
| --- | --- | --- |
| `tokio` | `1.53.1` | Process/I/O supervision and scheduling; explicit `rt-multi-thread`, `macros`, `io-std`, `io-util`, `process`, `sync`, `time`, `signal`. Bound worker counts and blocking work in configuration. |
| `tokio-util` | `0.7.19` | Cancellation/task lifetime; enable `rt`. |
| `rusqlite` | `0.40.2` | SQLite; disable defaults, enable `bundled`, `cache`. Bundle SQLite for controlled engine provenance and avoid a system SQLite requirement. |
| `sha2` | `0.11.0` | SHA-256 artifact identity; disable defaults for the fixed-output hashing API. |
| `uuid` | `1.26.1` | Run/session/feedback identity; enable `v7`, `serde`. IDs are distinct from source hashes and protocol request IDs. |
| `toon-format` | `0.5.0`, conditional | First codec candidate; **disable default features**. Admit only after BP4's format checks. |

SQLite calls are synchronous: place the connection on an owned, bounded database worker, with short transactions and no transaction spanning analyzer/model waits. Test with a real temporary SQLite store. `bundled` adds a C compilation prerequisite; it does not require users to run a database service. Rust toolchain compatibility for crates without a declared MSRV remains a build check.

Serialization order for artifact identities must be defined independently of incidental map iteration or Cargo feature unification. Hash retained bytes under a versioned identity contract; never hash whichever CLI rendering happens to be selected.

### C — Add with the corresponding external boundary

| Dependency | Exact candidate | Introduction trigger |
| --- | --- | --- |
| `oxc_allocator`, `oxc_ast`, `oxc_parser`, `oxc_semantic`, `oxc_span` | All `0.150.0` | TypeScript structural adapter; keep the five qualified libraries in sync and do not export Oxc ASTs from the adapter. |
| `lsp-types` | `0.97.0` | Shared TypeScript/rust-analyzer LSP client; no `proposed` feature initially. This supplies types, not a complete client lifecycle. |
| `reqwest` | `0.13.5` | First remote judge; disable defaults, enable `json`, `rustls`. Reuse one bounded client and retain exact transmitted/received bodies without credentials. |
| `schemars` | `1.2.2` | First cross-language bridge/session schema export; generate from real Rust types. |

The verified `reqwest` feature is named `rustls` in this release, and selects an AWS-LC-backed TLS stack with platform certificate verification. Its native build prerequisites must be checked when adding it; disabling default features does not make the dependency tree pure Rust. No provider-specific SDK is required by the proposed HTTP boundary. Generic HTTP retries must not bypass the engine's total model deadline or repeat ambiguous inference silently.

Do not preinstall additional parsers, an ORM, a plugin framework, a vector database or a benchmark framework. Git operations can initially use the installed Git executable with argument arrays and machine-readable output. An OS/process resource library may be justified when implementing descendant cleanup and whole-tree memory sampling; Tokio alone is not proof of either guarantee. Select and verify that dependency against the actual supported-host contract then.

## BP3 — Language packages and deployment boundaries

| Integration | Proposed pinned inputs | Boundary |
| --- | --- | --- |
| TypeScript | `typescript` **7.0.2**, including its platform-specific native package at the same version | Rust runs the native LSP directly when sufficient; a Node worker imports the pinned typed API only for structured queries requiring it. |
| PHP | Mago binary **1.48.1**, Composer SDK **`carthage-software/mago` 1.48.1**, locked **`revolt/event-loop` 1.0.9** | PHP worker imports the real SDK; Rust owns admission, lifetime, mapping and coverage. |
| Rust | Official rust-analyzer release **2026-09-14**, observed binary identity **0.3.3049-standalone** | Managed external LSP process; the adapter remains replaceable. |

These are the [qualified integration versions](../research/2026-09-22-language-qualification/README.md), not a recommendation to follow `latest`. npm and Packagist metadata were refreshed for the exact packages. The existing qualification used Node **24.18.0** and PHP **8.5.0**; retain these as the initial runtime test baseline, not an untested claim of support for every allowed engine version.

Use Bun for the small TypeScript package's dependency management, with a committed `bun.lock`; continue launching its worker explicitly with Node, as qualified. Bun as package manager does not establish Bun runtime compatibility. The initial `.mjs` worker follows the qualified JavaScript API and requires no bundler. Rust public-boundary tests drive both real workers; introduce another language-specific test framework only if substantial worker logic justifies it.

The bridge packages are tooling dependencies of Anti-Slop, never dependencies inserted into analyzed repositories. Resolve their installation directory independently of the current workspace. Do not rely on `/tmp` qualification paths or another project's `node_modules` in production. Supplying versioned binaries and packages is a later distribution step; normal analysis must not auto-install or auto-update them.

The Mago Composer executable inspected in the qualified source calls `ensure_binary` and can download an executable when first run. Invoke a separately provisioned, verified native Mago binary for analysis and use Composer for SDK loading; do not let an ordinary analysis trigger that installer.

Future installation uses Socket Firewall for supported Cargo/Bun fetches, exact candidates, lockfiles and release-age checks. Existing cached packages are not newly inspected by a network firewall; do not delete caches automatically. Composer requires the separately authorized uncovered-ecosystem path, with scripts/plugins disabled, reviewed registry sources, audit and version locking. Earlier permission covered temporary qualification only. No Composer command or binary download is requested by bootstrap A.

## BP4 — Codec qualification before claiming a TOON profile

D6 stays unchanged: TOON is the CLI default; explicit JSON represents the same projected report; JSONL is the dataset export. The canonical data contract remains Rust-owned.

`toon-format` 0.5.0 is a candidate, not yet an Anti-Slop-qualified codec. Its [published feature metadata](https://crates.io/api/v1/crates/toon-format/0.5.0) enables `cli` by default, pulling terminal UI, clipboard and tokenization functionality. Disable that feature. The [published source](https://docs.rs/toon-format/0.5.0/src/toon_format/lib.rs.html) gates the TUI on `cli`, although its introductory documentation still contains an obsolete placeholder description.

The [implementation README](https://github.com/toon-format/toon-rust) advertises specification v3.0; the [specification site](https://toonformat.dev/reference/spec) identifies v4.1 as its current Working Draft. Neither a Serde round trip nor the shared project name proves compatibility with v4.1.

Before choosing this codec for output, test the actual report shape, official conformance cases for the declared profile, independent expected outputs, numeric preservation, Unicode, escapes/control characters, empty/ragged rows, and honest counts/truncation. Compare decoded TOON and JSON semantics, including coverage and incomplete states. Re-encoding through the same library alone is insufficient. Measure encoding latency and bounded allocations on realistic report sizes.

Prefer a documented, verified common output subset if it meets the required report contract. Do not introduce an undocumented TOON dialect, silently substitute JSON for default output, or write a general codec merely to complete the bootstrap. A failure changes the codec choice before the public output contract is frozen; it need not block the unrelated Rust scaffold.

## BP5 — Local state, validation and implementation boundary

Proposed default workspace state is `.anti-slop/`: `records.sqlite3` and SQLite auxiliary files are authoritative retained data; `cache/` contains disposable facts/managed views; `logs/` contains disposable technical logs. An explicit data-directory override supports CI. Exclude the effective state, cache and export destinations from source discovery, even when they sit inside the analyzed repository. Keep explicit exports at the user's chosen destination; the data store remains authoritative under D12.

The root `.gitignore` excludes build output, bridge installations, local state, local corpus paths and actual secrets. Commit the package lockfiles, example configuration, migrations and controlled fixtures. No real private repository source or runtime dataset belongs in test fixtures automatically.

Keep production configuration defaults in `config`; validate unknown/invalid inputs and snapshot the effective configuration into run records. Example configuration documents those defaults rather than defining a second authoritative set. The existing architecture remains authoritative for the selected numerical budgets until implemented; test that effective defaults match those decisions.

Development profile: `debug = 1`, `incremental = true`; preserve compilation caches through RED/GREEN cycles. Start with standard release settings and measure before adopting LTO, allocator changes or panic-policy changes. Performance acceptance uses release builds and the whole owned process tree, including bridges, SQLite work and capture cost. No latency or memory guarantee follows merely from choosing Rust.

The approved **bootstrap A** creates the package/toolchain metadata, minimal library and binary entrypoints, developer documentation, ignore rules and example-configuration location; installs and locks group A; and validates compilation plus CLI help/version/error behavior. The user's implementation restriction leaves application behavior for a later instruction. Register only the scaffold's help/version surface: an unfinished analyzer must never appear as a successful clean scan. Reserve the target directories with `.gitkeep` markers without manufacturing unused source files or inventing application types before their contracts are implemented.

Groups B/C, bridge installations and substantive analysis behavior belong to subsequent implementation slices. Their destinations and candidates are proposed here so work can start without re-deciding ownership. Source capture, record integrity, reports/session lifecycle, language adapters, rule evidence and judges can be built in that order, with retention and explicit incomplete outcomes present before real judgments are produced. This ordering does not reduce the first usable release's three-language R1–R9 scope.

Focused contract coverage grows with those slices: real SQLite transactions/retention and self-contained export, protocol framing/backpressure/EOF cleanup, source-generation identity and stale-response rejection, semantic equivalence of output formats, and provider replacement without changing canonical records. Keep independent human labels distinct from provider predictions. The Pi-lens, quickcoin-platform and Zerobox performance corpus is opt-in and versioned at measurement time, separate from a reviewed rule-quality corpus.

The original proposal was checked through source/registry inspection and documentation consistency only. Bootstrap A adds dependency resolution and scaffold checks, recorded in the implementation report. Worker tests, codec conformance, native installations and representative performance runs remain deferred to their corresponding implementation boundaries.
