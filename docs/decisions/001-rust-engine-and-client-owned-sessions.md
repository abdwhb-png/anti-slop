# ADR-001: Rust engine and client-owned sessions

## Status

Accepted. The package scaffold exists; the engine and resident behavior are not implemented.

## Date

2026-09-22

## Context

Anti-Slop must analyze an agent's changes with repository context, both locally and in CI. Repeated analysis should reuse useful native state, including for a future Pi consumer. Separate implementations would diverge in rule behavior, policy and dataset capture. Low latency and controlled memory are primary reasons for choosing Rust.

## Decision

Use one Rust package, `anti-slop`, exposing the `anti_slop` library and the `anti-slop` executable. The approved scaffold uses edition 2024 and minimum Rust 1.96; the installed development pin and resolved dependencies are authoritative in `rust-toolchain.toml`, `Cargo.toml` and `Cargo.lock`.

Provide a one-shot CLI and an on-demand resident mode in the first usable version. Both call the same application engine. A resident process belongs to its client session and stops when the client closes; it is not a service shared across client lifetimes. The future Pi package will schedule analyses and consume reports through this boundary, without duplicating analysis, policy or dataset writing.

**D18: Support Linux first, including WSL2.** macOS and native Windows are later work. Verify process supervision, abrupt parent termination and whole-process-tree resource accounting on the supported Linux environments before claiming those guarantees. A successful graceful shutdown alone is insufficient.

The approved module layout assigns each shared behavior one owner:

| Module | Owns |
| --- | --- |
| `workspace` | Selected change scope, repository inputs, source generations and history access. |
| `analysis` | Native sessions, capabilities, language facts and normalization. |
| `rules` | Rule meaning, candidate selection, evidence and counterevidence requirements. |
| `judges` | Provider input rendering, transport and assessment normalization. |
| `policy` | Qualification and blocking decisions from supplied results/configuration. |
| `records` | Durable analyses, evidence, feedback, retention and export. |
| `engine` | Shared sequencing, budgets, cancellation and operation lifecycle. |
| `output` | One report projection and its encoders. |
| `session` | Resident transport and client lifetime. |
| `cli` | Arguments, dispatch and process exit behavior. |

Types live with the responsibility that owns their meaning. Native and provider types stop at their adapters. Shared transport/process mechanics may be reused where their contracts match; language semantics remain explicit. Introduce modules when implementing their responsibilities, rather than filling reserved directories with placeholder APIs.

Keep one package until a real independent consumer or build boundary justifies another crate. Exact deferred library candidates in the historical bootstrap proposal are not installed dependencies or permission to add them all.

## Alternatives Considered

- **One-shot execution only:** simpler lifetime management, but repeated startup defeats the selected interactive workload.
- **A global daemon across clients:** offers broader reuse but adds discovery, ownership and cross-client coordination that the first version does not require.
- **Separate CLI, CI and Pi engines or an initial multi-crate framework:** creates multiple maintenance paths before their independence is demonstrated.

## Consequences

The shared engine must expose explicit progress, cancellation, incomplete work and reusable reports. The session must own and clean up its descendants with bounded queues and shutdown. These are implementation obligations, not capabilities supplied by choosing Rust.

The first usable release still covers all languages and rule families in [ADR-003](003-evidence-first-rules-and-selective-blocking.md). A small implementation slice is a sequencing choice, not a release-scope reduction. Automatic code rewriting, a hosted collector and the Pi package itself remain outside this initial core scope.

## Evidence and Remaining Qualification

Consolidates D2, D4, D10 and D11 from the [historical design](../brainstorming-decisions/2026-09-22-anti-slop-design.md), approved BP1 from the [bootstrap proposal](../brainstorming-decisions/2026-09-22-anti-slop-bootstrap-proposal.md), and D18 selected during documentation preparation.

The [architecture proposal](../brainstorming-decisions/2026-09-22-anti-slop-architecture.md) describes a stdio protocol and lifecycle strategy. Exact commands, method names, framing/version compatibility, exit codes and supervision mechanisms still require a concrete public contract and tests. No production support claim follows from the initial scaffold.
