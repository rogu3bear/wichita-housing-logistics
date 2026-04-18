# wichita-housing-logistics

Internal operations app: **Intake → Assessment → Placement → Follow-up** for
Wichita housing logistics. Leptos 0.8 SSR + hydration on Cloudflare Workers
with D1. Forked from `rogu3bear/leptos-cloudflare`.

## Tech stack

- **Rust** + **Leptos 0.8** (single crate, feature-gated `ssr` / `hydrate`)
- **Cloudflare Workers** via `workers-rs` 0.7 + `axum` 0.8
- **D1** for persistence; prepared statements via `worker::D1Type`
- **cargo-leptos 0.3** + **wasm-bindgen-cli 0.2.114 (pinned)** + **worker-build 0.7**
- **Bun** (via `bunx wrangler`) for all Node tooling — never `npm`/`npx`

## Build / test commands

Prefer `cargo check` during iteration — edge builds are 5-15 min.

```sh
# Fast iteration
cargo check --features ssr
cargo check --lib --features hydrate --target wasm32-unknown-unknown

# Migrate local D1
bunx wrangler d1 migrations apply wichita-housing-logistics-db --local

# Full local build + serve
bash ./scripts/build-edge.sh
bunx wrangler dev --local --ip 127.0.0.1 --port 57581

# Remote deploy (after D1 IDs are populated in wrangler.toml)
bunx wrangler deploy
```

## Conventions

- **One crate**. SSR code is gated behind `#[cfg(feature = "ssr")]` and server
  functions use `send_wrapper::SendWrapper` around the SSR body because
  `worker::Env` is `!Send`.
- **Enums live in the database**. `stage`, `status`, `kind`, and `entity_type`
  are `TEXT` columns with `CHECK (… IN (…))` constraints. The Rust layer
  mirrors the allow-list in `ALLOWED_*` `&[&str]` constants and validates
  via `validate_allowed()` in `src/server/mod.rs` *before* the query runs.
- **Timestamps stamp server-side**. `updated_at` is always
  `CURRENT_TIMESTAMP` on mutation; `placement.started_at` / `ended_at` are
  auto-stamped only on specific status transitions (`moved_in`, `exited`,
  `cancelled`).
- **IDs cross the i32/i64 boundary**. Wire types expose `i64`, but
  `worker::D1Type::Integer` is `i32`. Convert through `row_id_arg(id)` in
  `src/server/mod.rs`, which also rejects overflow.
- **Error mapping**. Query layers return `AppResult<T>` (`AppError::Client` vs
  `AppError::Internal`). The `server_error` helper logs internal errors via
  `worker::console_error!` and returns a generic message to the client.

## Known gotchas

- **wasm-bindgen version drift.** `Cargo.lock` pins `wasm-bindgen 0.2.114`.
  The CLI must match or `cargo leptos build` will fail. If CI says
  "wasm-bindgen version mismatch", run:
  `cargo install -f wasm-bindgen-cli --version 0.2.114`.
- **Template de-templating is one-way.** `scripts/init.sh` refuses to run
  once real D1 IDs are in `wrangler.toml`. Don't run it again.
- **Placeholder D1 IDs are the deploy gate.** `wrangler.toml` ships with
  `00000000-0000-0000-0000-000000000000` for `database_id` and
  `preview_database_id`. First-time deploy must `wrangler d1 create` and
  paste the real IDs in.
- **`.github/workflows/rust.yml`** is a leftover from the template and runs
  plain `cargo build`, which doesn't select the feature split — it will
  fail on push to `main`. Either remove it or replace it with the template's
  intended verification flow (`scripts/check-deps.sh` + the two
  `cargo check` targets above). Out of scope for this PR per project policy.

## File-level map

See `README.md` for the project layout. Key entry points:

- `src/lib.rs` — `#[worker::event(fetch)]` → axum router → Leptos SSR
- `src/app.rs` — five routes, shared `<Router>` wrapper
- `src/api.rs` — every shared wire type and every `#[server]` fn
- `src/server/dashboard.rs` — composes the four entity modules
- `migrations/` — the authoritative schema

## Anti-goals

Explicitly out-of-scope until the MVP is in users' hands:

- Detail / edit pages (create + inline status transitions only).
- Authentication, authorization, or user management beyond a demo stub.
- AI-driven matching.
- Maps, geocoding, dispatch/rideshare metaphors.
- External system integrations (HMIS, VA, landlord portals).
- Event buses, websockets, microservices.

## Per-project git conventions

- Work on `main`.
- Never push / open PRs without explicit "done" / "ship it".
- Commit messages: imperative, lowercase, no trailing period.
- Scope prefixes encouraged: `feat(server):`, `feat(ui):`, `feat(db):`, `style:`, `fix(…):`.
