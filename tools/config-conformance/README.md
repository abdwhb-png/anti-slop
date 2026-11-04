# Local configuration verification tools

These packages serve tests only. The production CLI and generator are Rust.
The exact npm manifest/lockfile pins an independently implemented official
TypeScript TOON 4.1.1 decoder and VS Code's JSON language service 5.7.2 with
textdocument 1.0.15. Verify registry metadata, the resolved graph and seven-day
release ages before changing this lock, then fetch through Socket Firewall:

```sh
sfw npm ci --ignore-scripts --no-audit --no-fund
```

From the repository root, `node tools/config-conformance/schema.mjs` reads the
local generated schema and fixture set. It registers that content under the
reserved identity, disables remote loading and tests real completion, hover
descriptions and diagnostics using the headless editor-service APIs. It prints
observations to stdout and starts no service. It does not claim GUI/editor
settings installation or public availability of the schema URL.

`decode.mjs` accepts TOON on stdin and emits independently decoded JSON. Rust
CLI/output tests invoke it to check the declared inspection projection domain;
this is not a round trip through the production Rust codec.
