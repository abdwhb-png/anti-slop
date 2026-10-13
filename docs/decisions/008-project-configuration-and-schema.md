# ADR-008: Project configuration and schema

## Status

Accepted for the JSONC format, linked schema, initial root configuration and compatible evolution toward subproject configurations. Configuration loading, schema generation/publication and migration tooling are not implemented. The future O2 feature is planned, not part of the first usable release.

## Date

2026-09-22

## Context

The user wants an editing experience similar to Oxlint, explicitly prefers JSONC over TOML, and requires a schema link rather than only a `schema_version` field. The initial organization must remain simple while its limitations and path toward independently configured subprojects are explicit.

The scaffold installed a TOML parser and the historical bootstrap sketched `anti-slop.toml`, but no configuration loader or user-facing configuration contract exists. This decision replaces that format proposal without changing the historical dependency record. It does not change the report and dataset encodings in [ADR-006](006-output-formats-and-shared-projection.md).

## Decision

### D23 — One root configuration with path overrides (O1)

Use one version-controlled `.anti-slop.jsonc` at the selected workspace root in the first version, with an explicit configuration-path alternative. Discovering the configuration does not select the changes to analyze: [D19](../PRD.md#us-01--select-exactly-what-is-analyzed) still requires a scope on every analysis request.

The file declares shared project settings and ordered path overrides. O1 does not discover nested configurations or implement `extends`. Its configuration explanation must identify the root file, effective settings and the source of each applied override. CLI, CI and resident clients use the same Rust resolver.

Use these resolution boundaries when specifying the executable contract:

- Resolve path patterns relative to the file that declares them. For O1 that is the root configuration. The process working directory must not change their meaning once the workspace/configuration is selected.
- Apply root rule settings before matching overrides, in declaration order. A later assignment replaces the entire earlier value for that rule; unrelated rule entries remain. Do not introduce implicit deep merging of rule options or array concatenation.
- Keep workspace-wide controls, including model-phase and storage budgets, outside per-file rule overrides. A matching override cannot reset a whole-analysis limit or grant source-transmission permission.
- Accept line and block comments and trailing commas. Reject duplicate object keys, unknown fields/rules, invalid values and unsupported configuration contracts with actionable locations. A newer O2 field must not be silently ignored by an O1-only binary.
- Keep analysis selection, source/context access and transmission controls distinct. A rule override is not a source-transmission permission or an instruction to discard the context needed to evaluate a cross-file rule.

Exact setting names, glob syntax, rule options and multi-file finding attribution remain implementation gates below. The earlier illustrative `off` / `advisory` / `block` vocabulary is not yet a published enum. Whatever names are selected, requesting blocking never establishes the qualification required by [ADR-003](003-evidence-first-rules-and-selective-blocking.md).

### D24 — JSONC with a versioned schema link

The project file carries a `$schema` property containing a real, published, version-pinned HTTPS URL for its JSON Schema. Published examples must include that working URL; a standalone `schema_version` value does not fulfill this editing requirement. Do not require users to maintain a redundant version field solely for schema association.

The schema must provide completion, descriptions, allowed values and structural validation for the supported contract. Generate the published artifact from the authoritative configuration definition used by the Rust implementation; keep defaults and rule options under their actual owner rather than maintaining another handwritten specification. Schema generation technology remains to qualify.

Separate editor assistance from runtime behavior:

- The editor may fetch the linked schema. Publish immutable versioned artifacts and retain old links; generated examples must not depend on a moving branch or a `latest` alias.
- The Rust engine validates against its supported local contract and does not fetch `$schema` or remote references during configuration loading. Editor connectivity must not become an analysis-startup dependency.
- Record the effective contract/schema identity with the run. Define how schema URLs map to supported contracts and how unsupported revisions are diagnosed before publishing the loader; a URL cannot override the binary's validation rules.
- Structural validation does not qualify a rule for blocking, authorize remote inference or certify an analyzer's capabilities. Those checks retain their existing owners.

The publication origin and exact URL are unresolved. No Git remote is configured in this checkout, and no Anti-Slop schema has been published by this task. Resolve the publication location before shipping editor support; do not present an illustrative or guessed address as a working endpoint.

### D25 — Document O1 limits and preserve an explicit path to O2

O1 supports different rule settings for paths in a mixed-language repository. It does not provide independently owned configuration units or reusable configuration inheritance. Its scaling limits are organizational and semantic; no measured file-count threshold has been established.

| Limit | When it matters | O2 response |
| --- | --- | --- |
| L1 — Central ownership | Several teams repeatedly modify the same root file or need to review their settings alongside their subproject. | Let each explicitly participating subproject own its configuration. |
| L2 — Overlapping overrides | Understanding one file's rules requires tracing many ordered exceptions; moving a directory changes several patterns. | Keep local adjustments near their scope while retaining an inspectable resolution order. |
| L3 — No reusable inheritance | Shared rule settings must be repeated across independently configured projects. | Introduce explicit `extends` of declarative shared configuration files. |
| L4 — Cross-project evidence | A rule relates files with different overrides, languages or ownership. Splitting configuration alone cannot settle the rule's scope or reporting policy. | Preserve whole-analysis evidence and define attribution/conflict behavior before enabling blocking for that case. |

Revisit O2 when independent ownership or repeated override/sharing problems are demonstrated. A large repository alone does not require O2. O1 does not promise per-subproject budgets, stores, judge processes or authorization domains.

The future O2 design must satisfy these compatibility obligations:

1. **Explicit adoption.** Existing O1 files retain their meaning. An upgrade must not start loading nearby configurations automatically. Subproject participation is declared explicitly by the selected root configuration; the field names and version transition belong to the future O2 contract.
2. **One contract and resolver.** Child files use JSONC, linked schemas and the same rule-setting representation. Extend the existing configuration owner with bounded composition rather than adding separate CLI, CI or Pi resolution behavior. O1 needs no generic plugin or inheritance framework in advance.
3. **Explicit sharing.** A child inherits shared settings through `extends`, not proximity alone. Resolve references and patterns relative to their declaring files; specify deterministic ordering and reject cycles, missing references and incompatible contracts. Initially plan local, explicitly supplied files; network package resolution is a separate future decision.
4. **Preserved ownership.** Keep session-wide resource limits and trusted authorization outside child rule inheritance. An inherited rule policy still cannot manufacture blocking qualification. Configuration partitions must not silently remove cross-project evidence.
5. **Reviewable migration.** Before moving settings, capture the effective rules and origins for the affected paths. Rebase path patterns and compare before/after resolution, including overlapping overrides, unmatched paths and multi-file cases. Preserve historical configuration identities; requalify any profile whose effective behavior changes. Exact migration commands or automation are future work.
6. **Qualified compatibility.** Retain O1 fixtures when implementing O2. Prove old configurations behave identically, new fields fail explicitly on unsupported versions, and mixed inherited/overridden settings have stable provenance and bounded resolution cost.

These obligations prepare the expansion; implementing nested discovery or `extends` in the first release is not required.

## Alternatives Considered

- **O1 — Root configuration with overrides:** selected for a predictable initial setup. Its ownership, overlap and reuse limits are recorded above; it remains supported when O2 is introduced.
- **O2 — Subproject configurations with explicit sharing immediately:** useful for independent teams, but adds composition and migration semantics before a demonstrated need. Planned as an explicit compatible extension.
- **O3 — Executable configuration:** dynamic setup adds runtime execution and reproducibility requirements. No current configuration need justifies it; JSONC remains declarative.
- **TOML or multiple initial formats:** TOML matched the historical bootstrap, but the user selected JSONC's editing experience. Supporting two formats would add parser/documentation qualification without a present requirement.
- **Only a version number or a moving schema URL:** fails the selected schema-linked editing experience or permits editor behavior to drift without a reviewable configuration change.

## Consequences

Resolve configuration once per analysis generation and reuse validated state in a resident session while its inputs remain unchanged. Bind each run to an immutable effective configuration and source provenance, excluding credential values. Configuration changes must not mutate an in-flight run; invalidate affected facts, assessments and qualifications before publishing a new current result. Inspection surfaces must explain effective values and origins without revealing secrets.

Repository configuration expresses shared policy. Credential storage and trusted repository transmission authorization remain governed by [PRD OPEN-03](../PRD.md#open-specifications-and-release-gates); a checked-in file cannot self-authorize disclosure. Preserve explicit scope, model replacement, whole-analysis budgets and dataset lineage under their existing decisions.

The installed `toml` dependency remains a record of the scaffold, not a supported project format. Review its removal and the JSONC/schema dependencies during the authorized implementation task, using the dependency workflow. This documentation change neither installs dependencies nor adds a configuration parser, schema, CLI command or schema-hosting service.

## Evidence and Remaining Qualification

The user selected D23, then explicitly selected JSONC with a schema link and requested documented O1 limits plus planned O2 evolution on 2026-09-22. The local CLI currently implements help/version only; the [root README](../../README.md#current-status) remains authoritative for shipped behavior.

Oxlint documents declarative configuration, path-specific overrides and explicit shared configuration in its [configuration guide](https://oxc.rs/docs/guide/usage/linter/config). Its [nested configuration documentation](https://oxc.rs/docs/guide/usage/linter/nested-config) distinguishes nearest-file selection from explicit inheritance. Anti-Slop adopts the useful editing conventions while choosing explicit O2 participation. VS Code documents [JSONC and schema association](https://code.visualstudio.com/docs/languages/json#_json-schemas-and-settings), including `$schema` for completion and validation; this is editor behavior, not a promise that arbitrary remote schemas control Anti-Slop.

Before publishing configuration support, resolve and verify:

- The public schema origin, immutable URL/revision policy, JSON Schema dialect, generator and parity with runtime validation. Exercise completion/descriptions and invalid settings in the supported editor workflow, including offline runtime validation.
- The JSONC parser's comment/trailing-comma behavior, duplicate-key errors, source locations and size/depth limits; no parser is qualified by this ADR.
- Exact root/config discovery, defaults, setting and rule identifiers, allowed request/CLI overrides, glob semantics, unsupported-contract errors and path/symlink boundaries. Freeze these in the implementation contract and keep the effective view reproducible from any entrypoint.
- Rule-specific attribution when a finding spans multiple configured paths. Do not select a permissive or restrictive policy merely from the first file encountered, and do not count unresolved policy coverage as a clean required check.
- Schema/example/runtime conformance, overlapping-override cases, configuration changes during residency, provenance/export fidelity and the applicable [resource budgets](007-performance-budgets-and-qualification.md).

O2's final composition schema, limits and migration tooling are gated by a future O2 implementation task, not by the first O1 release.
