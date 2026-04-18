#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

# Build-time identity: short commit SHA so the deployed footer can say
# which revision is live. Falls back to "unknown" outside a git tree.
export GIT_COMMIT_SHA="${GIT_COMMIT_SHA:-$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)}"

# Stable server-fn URLs. By default the #[server] macro appends an xxh64
# of (CARGO_MANIFEST_DIR + module_path) to each route to disambiguate
# cross-crate name collisions. We have one crate with unique fn names, so
# the hash is pure friction — it makes curl tests brittle and obscures
# the URL in logs. Strip it.
export DISABLE_SERVER_FN_HASH=1

cargo leptos build --release
bun ./scripts/hash-assets.mjs
source "$ROOT_DIR/target/asset-hashes.env"
if ! command -v worker-build >/dev/null 2>&1; then
  cargo install -q "worker-build@^0.7"
fi
worker-build --release --features ssr
bun ./scripts/verify-hashed-assets.mjs
