#!/usr/bin/env bash
# Run from any directory after the locked dependencies have been fetched.
set -euo pipefail
ci_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ci_root"

ci_base="${CI_BASE_SHA:-}"
if [[ -z "$ci_base" ]]; then
  ci_base="$(git rev-parse --verify HEAD^ 2>/dev/null || true)"
fi

cargo fmt --all -- --check
python3 -m unittest discover -s scripts/ci -p 'test_*.py' -v
python3 scripts/ci/policy.py whitespace --base "$ci_base"
cargo build --frozen --workspace
cargo test --frozen --workspace --all-targets
cargo test --frozen --workspace --doc
cargo clippy --frozen --workspace --all-targets -- -D warnings
cargo run --frozen --example config_schema -- --check
node tools/config-conformance/schema.mjs
