#!/usr/bin/env bash
# seed-local.sh — load reviewable fixtures into the LOCAL D1 only.
#
# Production D1 starts empty. Run this before `wrangler dev` if you want
# a populated UI for iteration.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB="wichita-housing-logistics-db"

echo "[seed-local] applying scripts/seed-local.sql to $DB (--local)"
bunx wrangler d1 execute "$DB" --local --file="$ROOT_DIR/scripts/seed-local.sql"
echo "[seed-local] done"
