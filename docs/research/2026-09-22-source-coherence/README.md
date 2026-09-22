# Source-generation coherence experiment

Date: 2026-09-22

Status: 20 focused checks passed with the previously qualified native binaries. This experiment supports a managed source view as the reference implementation path. It does not qualify production snapshot consistency or the newly selected performance budgets.

## Question and observed result

Can an analyzer continue to assess a captured source generation while the original working files change, and can a subsequent explicit update reach dependent analysis? Can a disk-only manifest miss an in-memory semantic input change?

| Observation | Evidence | Implication |
| --- | --- | --- |
| SC1 — TypeScript isolation and update | With native TypeScript 7.0.2, the managed copy continues to return `const result: OrderId` after the original file changes. An explicit update yields `const result: number`. A newly introduced caller appears in a new reference query. | Separate the analyzed source view from live edits; invalidate reference-query scope when files are added. |
| SC2 — Rust isolation and update | With rust-analyzer 2026-09-14, the dependent `OrderId` layout remains eight bytes while only the original file changes, then becomes four bytes after an explicit update. The build-script marker remains absent. | Reuse a loaded session across controlled generations without enabling build-script execution. |
| SC3 — PHP capture | Mago 1.48.1's real PHP SDK reports the unchanged copied interface after the original changes. The next run reports the updated interface source and real type-mismatch diagnostics with nonzero exit. | SDK source retention follows the managed files; diagnostics must remain visible when a new generation is invalid. |
| SC4 — Disk-only verification is insufficient | In both LSP experiments, a `didChange` message changes dependent facts while every file in the recorded disk manifest remains unchanged. | Include exact overlay content/version in input identity; disk hashes alone cannot identify the bytes analyzed by an LSP session. |

The successful prototype uses independent ordinary copies, not hard links or symlinks back to mutable source. Each LSP adapter is controlled by this experiment and uses one sequential query stream. The final source update agrees with both the recorded bytes and the native response.

## Scope and exclusions

The inputs are the small generated fixtures from the [initial qualification](../2026-09-22-language-qualification/README.md). Source manifests cover the declared `.ts`, `.json`, `.rs`, `.toml`, and `.php` fixture files, excluding native `target` output. The capture JSON retains the actual text and its SHA-256 for each tested state. Native standard libraries and toolchains are reused; this is not a hermetic capture of every dependency.

The checks cover eight TypeScript observations, eight Rust observations, and four PHP observations, including ordinary LSP shutdown. They do not establish:

- X1 — Configuration/dependency changes, escaped symlink targets, Cargo feature reloads, framework-generated sources, procedural macros or mutable external packages are fully captured.
- X2 — An analyzer query racing an update, delayed responses, a client crash, or an abruptly killed process tree is handled by a production coordinator.
- X3 — A filesystem-wide atomic capture, or perpetual freshness after publication. A manifest describes a captured input set, not a frozen live filesystem.
- X4 — Copying or maintaining representative projects meets B1–B4. There is one session per LSP analyzer and two PHP invocations, not a performance sample suitable for p95 qualification.

The inference is therefore limited: a managed source view is a viable reference mechanism for these fixtures; the remaining source-consistency contract still needs adapter-specific tests. Native overlays can be used only when their contents are bound to the same generation as configuration and dependencies.

## Provenance and reproduction

[probe.py](probe.py) imports the real previously qualified LSP transport and memory helper, verifies those helpers against the retained sources, imports the canonical fixture generator from the repository, and invokes the canonical PHP SDK worker. It verifies the three analyzer binary hashes against the original [provenance](../2026-09-22-language-qualification/provenance.json). No package was installed and no user repository or remote model was analyzed.

A setup-only first attempt stopped before analyzer execution because the old temporary fixture-generator copy differed from the retained version. The final probe uses the retained generator directly; it does not weaken the transport or binary identity checks.

Given the existing qualified runtime/dependency directory, run with a new output directory:

```sh
python3 docs/research/2026-09-22-source-coherence/probe.py \
  --qualification-root /tmp/anti-slop-language-qualification-20260922 \
  --output /tmp/anti-slop-coherence-new-run
```

Reprovisioning that runtime, if absent, is a separate dependency operation subject to the installation policy. See the original qualification's replay/setup instructions; this probe does not download anything.

[results.json](results.json) retains checks, raw queried facts, commands and binary identities. [evidence.tar.gz](evidence.tar.gz) additionally retains all captured source versions, raw LSP exchanges, PHP SDK records and process output. The archive excludes executables, dependency trees and build output. Its raw paths identify the temporary experiment; no private project source was collected. `artifact-manifest.json` fingerprints the retained report and evidence files.

Validation: the live probe completed with all 20 checks passing; dependent hover values and PHP generation/error output were inspected separately. These are source-coherence observations, not rule-accuracy labels or an Anti-Slop benchmark.
