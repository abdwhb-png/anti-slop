# AS-M002 recovery for review

Recovered on 2026-11-04 from the original implementation and independent
validator chats after the uncommitted `/workspace/anti-slop-as-m002` worktree
and `.missions/AS-M002` directory disappeared. The user authorized creating a PR.

The branch restores the shared JSONC configuration resolver, path overrides and
origins, read-only `config explain` in TOON/JSON, Rust-derived JSON Schema,
fixtures and headless editor-service checks. It does not implement analysis.

## Provenance

- Original implementation chat: `01a13795-2c40-74a2-ab9d-84a8dfb6d077`,
  turn `01a139e5-45a7-7513-ab37-3ea78c57a1b8`.
- Original independent validator: `01a13a6b-417c-73b2-ab16-48297fb6d04b`.
  Its complete final report is preserved in
  [historical-validator-report.md](historical-validator-report.md).
- [contract-r4.txt](contract-r4.txt) preserves the complete original contract
  assembled from overlapping captured reads. It is historical evidence, not a
  new active mission revision. Relative links inside that capture retain their
  original locations.
- Rust edits were replayed from original file-change records and local source
  transformations, then checked against original validator reads where available.
  [source-comparisons.json](source-comparisons.json) records complete comparisons.
- The original Cargo lock patch, generated schema, fixture JSON and npm lockfile
  were recovered from complete original validator outputs. The recovery commit
  starts from current `main` (`cdc3aca`), rather than the original unavailable
  worktree base `96a2521d3bb024361f31fdc788e957f7c63c49bb`.

## Fresh verification

[checks.json](checks.json) records current build, all four integration test
targets, schema regeneration, formatting, Clippy and the real local editor-service
check. Per-command outputs are retained alongside it. The Cargo and npm registry
checks record release ages and matching checksums/integrity before package
downloads. All 23 added Cargo packages and seven npm packages were checked.
Downloads used `sfw`; the original scaffold's Cargo package tuples are preserved.

Historical acceptance does not establish independent acceptance of this recovery
commit. This PR remains a draft for that review.

## Recovery limits

The original complete state/history, packet files, rejected candidate archives,
boundary manifests and detailed qualification evidence have not been recovered.
No replacement completion state or fabricated journal has been created. Do not
treat this directory as the complete original mission archive, or the original
review as proof that every recovered byte is identical.

The original reviewer explicitly retained a release-age control violation:
`smallvec 1.16.3` was fetched after an age check rejected it. The accepted final
lock and this recovery use `1.16.2`. Fresh checks cannot repair that historical
incident. The schema URL remains reserved and unpublished. Public schema hosting,
GUI editor setup, native performance and AS-M003 remain outside this PR.
