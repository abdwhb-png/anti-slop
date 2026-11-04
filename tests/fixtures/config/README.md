# Shared structural conformance cases

The expected booleans in `conformance.json` are independently specified from
AS-M002 r4: supported identity, root/override fields, nine rule identifiers and
three scalar policy values. Rust tests feed these documents through the actual
resolver; the local VS Code JSON language-service harness feeds the same cases
through the generated draft 2020-12 schema.

Duplicate decoded keys, JSONC dialect, source-byte/depth bounds, combined pattern
counts, UTF-8 byte limits, glob semantics and filesystem selection stay runtime
checks. Schema `maxLength` counts characters, not bytes; per-array pattern and
override counts are structural constraints. Matching semantics are exercised by
the real config and CLI tests, not a duplicated matcher in these fixtures.
