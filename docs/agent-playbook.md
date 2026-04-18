# Agent Playbook — wichita-housing-logistics

Instruction set for a new operator (human or AI) picking up this repo. Read
the root `CLAUDE.md` first — this file is the concrete command sequence.

The critical assumption inherited from the upstream template has changed:
**do not `export CLOUDFLARE_API_TOKEN=…`** anywhere outside the rotation
script. The minter (`CF_DEV_TOKEN`) lives only in `~/dev/.env`; every deploy
operation uses a short-lived scoped child minted by
`scripts/rotate-cf-tokens.sh` and sourced per-command in a subshell.

---

## 1. Prerequisites

```sh
./scripts/check-deps.sh      # every line must be [ok]
# if anything is missing:
./scripts/bootstrap.sh       # installs cargo-leptos, wasm-bindgen-cli 0.2.114
```

Verify the minter is present (value stays redacted):

```sh
grep -c '^CF_DEV_TOKEN=' ~/dev/.env
grep -c '^CLOUDFLARE_ACCOUNT_ID=' ~/dev/.env
```

Both must print `1`. If missing, mint a new account-owned token
(`Account API Tokens: Write` + `Account Settings: Read`) at
**Cloudflare dashboard → Manage Account → Account API Tokens**, paste into
`~/dev/.env`, done.

---

## 2. Mint scoped children

```sh
# The script refuses to run if CLOUDFLARE_API_TOKEN is already exported
# — that would be an alias violation. Unset it first if your shell has one.
unset CLOUDFLARE_API_TOKEN
bash ./scripts/rotate-cf-tokens.sh
bash ./scripts/rotate-cf-tokens.sh --list    # sanity: both children active
```

Produces two per-purpose env files under `.cf-tokens.local/` (gitignored,
mode 600):

- `whl-worker-deploy.env` — `Workers Scripts Write` + `Account Settings Read`, 90 d expiry
- `whl-d1-migrate.env`    — `D1 Write` + `Account Settings Read`, 30 d expiry

Each contains a fresh `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID`
scoped to one surface. **Source them inside a subshell per command** — never
let a child token leak into the interactive shell.

Revoke every tracked child (safe to re-run if you need a clean slate):

```sh
bash ./scripts/rotate-cf-tokens.sh --revoke-all
```

Partial failures leave the non-revoked children in the state file so a
retry can target them; the script exits 1 with the failed ids listed.

---

## 3. Apply D1 migrations remotely

`wrangler.toml` already ships with the real `database_id`
(`29c3ccfd-b580-4079-8a98-94c94c223c0a`, region WNAM). On a fresh clone
you do not run `wrangler d1 create` — the DB already exists.

```sh
( set -a; . .cf-tokens.local/whl-d1-migrate.env; set +a
  bunx wrangler d1 migrations apply wichita-housing-logistics-db --remote )
```

`scripts/seed-local.sql` is **not** a migration — it never runs against
remote D1. If you want local fixtures for a `wrangler dev` session, run
`bash ./scripts/seed-local.sh` against the local database.

---

## 4. Build and deploy

```sh
bash ./scripts/build-edge.sh   # cargo leptos + worker-build; no CF token needed

( set -a; . .cf-tokens.local/whl-worker-deploy.env; set +a
  bunx wrangler deploy --dry-run       # validates bundle + bindings
  bunx wrangler deploy )
```

Deploy prints a `*.workers.dev` URL and a Version ID. Note them; the
`BuildFooter` in the deployed UI will show `v<CARGO_PKG_VERSION> · <short
git sha>` for verification.

---

## 5. Smoke test

```sh
URL=https://<name>.<subdomain>.workers.dev
curl -sSI $URL/                                                      # 200, cache-control: private, no-cache
curl -sSI $URL/pkg/wichita-housing-logistics.*.wasm                  # 200, immutable
curl -sS  $URL/ | grep -oE 'v[0-9.]+ · [0-9a-f]+' | head -1          # build footer
```

If the footer shows `· unknown` for the SHA, the build did not pick up
`GIT_COMMIT_SHA` — re-run `scripts/build-edge.sh` from a git working tree
and redeploy.

---

## 6. Troubleshooting

| Symptom | Diagnosis / Fix |
|---|---|
| `rotate-cf-tokens.sh` exits with "CLOUDFLARE_API_TOKEN is set…" | Unset it. The script refuses to run when an ambient deploy token is in the shell — that's the alias-violation guard. |
| `wrangler d1 … --remote` returns 401 | Scoped child expired. `bash ./scripts/rotate-cf-tokens.sh` mints fresh. Expiry: 90 d worker-deploy, 30 d d1-migrate. |
| `wasm-bindgen version mismatch` at build time | CLI and lockfile drift. `cargo install -f wasm-bindgen-cli --version 0.2.114` rebinds. |
| Hydration silently broken — forms inert in the browser | Check that SSR HTML references `/pkg/wichita-housing-logistics.<hash>.{js,wasm,css}` and not `/pkg/.<hash>.*`. If the latter, `scripts/hash-assets.mjs` failed to export `LEPTOS_OUTPUT_NAME`. |
| `Internal error` at the client | `bunx wrangler tail` streams Worker `console_error!` — every `AppError::Internal` in server-fns logs before returning the generic `"Request failed. Try again later."` to the client. |
| `Cannot use the access token from location: <ip>` | IP-restricted minter. Add the current egress IP (`curl -s https://ifconfig.me`) to the token's allowlist in the dashboard. |

---

## Token doctrine (summary)

- `CF_DEV_TOKEN` lives in `~/dev/.env`. Only `scripts/rotate-cf-tokens.sh`
  reads it. Grep-guard: any other reader is a bug.
- Per-surface scoped children live in `.cf-tokens.local/` (gitignored,
  mode 600). Source in a subshell, never export globally.
- CI never mints and never deploys — the workflow at
  `.github/workflows/rust.yml` is check-only.
- Permission-group GUIDs are looked up at runtime; never hardcoded.
- Runtime token count for this Worker is zero — the Worker reads D1 via
  the `DB` binding, not via the REST API.

See `~/.claude/CLAUDE.md` "Cloudflare Token Doctrine" for the authoritative
invariants.
