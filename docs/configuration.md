# Local configuration inspection

Run `anti-slop config explain --workspace <dir> --path <relative-path>` to
inspect a future or existing path without reading its source or writing files.
`--config <file>` selects one file relative to the workspace or an absolute file.
No parent or nested configuration discovery occurs. Patterns use the resolved
declaring file's directory, including a selected configuration symlink target.
The target is normalized lexically inside the workspace; it need not exist.

An absent root `.anti-slop.jsonc` uses all nine documented advisory defaults,
with built-in origins and no captured configuration bytes. Explicitly missing,
unreadable, dangling or invalid files fail. Existing files require the exact
reserved identity:

`https://github.com/abdwhb-png/anti-slop/releases/download/config-v1.0.0/anti-slop.schema.json`

This asset is **unpublished**. Runtime identity validation is offline and never
fetches it. The generated local artifact is `schemas/anti-slop.schema.json`; public hosting
and release availability remain separate gates.

The supported fields are `$schema`, optional `rules`, and optional `overrides`.
`rules` maps R1–R9 to scalar `off`, `advisory` or `block` requests. An override
has a nonempty `include` glob array, optional `exclude`, and required `rules`.
Defaults apply first, then root rules, then every matching override in order.
Each assignment replaces the whole policy and preserves unrelated rules.
The result identifies every winning origin and the ordered matching indexes.
Includes match with any pattern; any exclusion prevents only that override.

Globs are case-sensitive UTF-8 with `/` separators: `*` within a segment, `?`
for one Unicode scalar, complete `**` for zero or more segments, classes/ranges
(`[abc]`, `[a-z]`, `[!abc]`), and nonnested brace alternatives (`{ts,tsx}`).
`**/*.ts` includes root files; recursive segments retain their semantics inside
alternatives. Dotfiles are ordinary names; backslash is literal. Re-inclusion
via leading `!`, partial `**`, malformed classes and nested braces fail.

JSONC line/block comments and trailing commas are accepted. Duplicate decoded
keys, JSON5 extensions, unknown fields/rules/options and schema aliases fail.
Bounds: 1 MiB input, depth 64, 256 overrides, 1,024 total include/exclude patterns,
4,096 UTF-8 bytes per pattern or inspected path. Errors are located and redact
rejected values. Returned snapshots retain actual input bytes and SHA-256;
later file edits cannot mutate earlier settings or provenance.

TOON is the default, including redirected stdout. `--format toon|json` selects
explicitly; both encode one shared projection. The pinned TOON **4.1 inspection
profile** uses comma, two spaces, stable fields and rule order, nested objects,
arrays, null, Unicode/control strings and bounded integer location metadata.
It is tested against an independent TypeScript decoder and retained reference
examples. This does not qualify a universal codec or an analysis report.
Structured results and errors use stdout; diagnostic failures use stderr.
Exits are 0 for a view/defaults, 2 for invalid arguments/configuration/identity,
and 3 for required I/O failure. Invalid format selection uses a TOON error.

Configuration identifies requested policy. It cannot authorize transmission,
select analysis scope, qualify blocking or make analysis available. Per-path
disagreement stays visible; multi-file finding policy remains deferred.

## Local schema and editor checks

The shared Rust input types generate draft 2020-12 schema; field names, unknown
field rejection, rule inventory and scalar policies have one owner. Regenerate and check from the repository root:

```sh
cargo run --frozen --example config_schema > schemas/anti-slop.schema.json
cargo run --frozen --example config_schema -- --check
```

Install the pinned verification tools through `sfw npm ci --ignore-scripts
--no-audit --no-fund` in `tools/config-conformance`, then run
`node tools/config-conformance/schema.mjs` from the repository root. This
registers the local artifact under its reserved identity in the actual VS Code
JSON language service, disables remote schema requests, and exercises JSONC
completion, descriptions/hover and invalid-value diagnostics. The recorded
qualification uses the headless editor-service API, not a GUI or an installed
editor setting. Editors must associate the local content explicitly until the
release asset exists; the URL alone cannot load an unpublished schema.

Twenty structural fixtures are checked by both the real resolver and this
service. The schema also constrains nonempty include lists, per-list pattern
counts and 256 overrides. Duplicate decoded keys and JSONC dialect, source
byte/depth limits, combined pattern counts, glob matching and filesystem/path
selection remain runtime checks. Schema string length counts characters; the
resolver independently enforces UTF-8 bytes. Local conformance does not prove
public availability or schema/runtime parity for those semantic checks.
