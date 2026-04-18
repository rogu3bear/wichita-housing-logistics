# Wichita Housing Logistics

Internal operations web app for coordinating housing logistics in Wichita.
Case managers, providers, and admins track households through the pipeline:

> **Intake → Assessment → Placement → Follow-up**

Built on Leptos 0.8 SSR + hydration, deployed to Cloudflare Workers with D1.
Forked from [`rogu3bear/leptos-cloudflare`](https://github.com/rogu3bear/leptos-cloudflare);
see `docs/` for the template's deeper CF deployment notes.

## Pages

| Route | Purpose |
|---|---|
| `/` | Operations dashboard — pipeline counters + recent activity |
| `/households` | Household roster + stage transitions |
| `/inventory` | Housing resources (shelter beds, units) + status transitions |
| `/placements` | Household × resource placements + lifecycle transitions |
| `/activity` | Polymorphic audit trail with note creation |

## Domain model

- **household** — the primary pipeline entity (head name, household size, stage)
- **housing_resource** — a placement slot with kind + capacity + status
- **placement** — links a household to a resource through proposed →
  confirmed → moved_in → exited (or cancelled)
- **activity_note** — polymorphic audit trail against household, resource,
  placement, or system

SQL lives in `migrations/`. CHECK constraints mirror the Rust allow-lists in
`src/server/*.rs`.

## Local development

```sh
# One-time toolchain (see scripts/check-deps.sh for full list)
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install -f wasm-bindgen-cli --version 0.2.114   # pinned

# Migrate local D1 and seed fixtures
bunx wrangler d1 migrations apply wichita-housing-logistics-db --local

# Type-check (fast)
cargo check --features ssr
cargo check --lib --features hydrate --target wasm32-unknown-unknown

# Full local build + serve
bash ./scripts/build-edge.sh
bunx wrangler dev --local --ip 127.0.0.1 --port 57581
```

## Deploying

1. Create the D1 database:
   ```sh
   bunx wrangler d1 create wichita-housing-logistics-db
   ```
   Paste the `database_id` into both `database_id` and `preview_database_id`
   in `wrangler.toml`.
2. Apply migrations remotely:
   ```sh
   bunx wrangler d1 migrations apply wichita-housing-logistics-db --remote
   ```
3. Deploy:
   ```sh
   bunx wrangler deploy
   ```

The `wrangler.toml` `[build]` hook runs `scripts/build-edge.sh`, which
compiles the Leptos crate, hashes the client assets, and runs `worker-build
--release --features ssr`.

## Project layout

```text
migrations/
  0001_init.sql          # households, housing_resources, placements, activity_notes
  0002_seed.sql          # reviewable fixtures
src/
  api.rs                 # shared wire types + #[server] fns
  app.rs                 # router with five routes
  asset_hashes.rs        # hashed-asset pointer (filled at build time)
  lib.rs                 # Worker fetch handler + hydrate() entry
  main.rs                # stub
  components/
    layout.rs            # TopNav, PageHeader, ErrorBanner, humanize()
    dashboard_page.rs    # pipeline counters + recent activity
    households_page.rs
    inventory_page.rs
    placements_page.rs
    activity_page.rs
  server/
    mod.rs               # AppError, server_error, helper validators
    state.rs             # AppState (LeptosOptions + Arc<worker::Env>)
    households.rs
    resources.rs
    placements.rs
    activity.rs
    dashboard.rs         # composes the other server modules
style/main.css           # ops palette (no framework)
scripts/                 # template bootstrap + build pipeline
```

See `CLAUDE.md` for coding conventions and `docs/` for the template's CF
deployment deep-dives.

## License

MIT — inherited from the template.
