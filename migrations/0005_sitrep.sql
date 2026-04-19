-- Sitrep (situational report) banner state. Single-row table — the
-- CHECK(id = 1) enforces that; INSERT OR IGNORE seeds the default
-- "inactive" row so every GET has something to return.
--
-- active=0 → <SitrepBanner/> renders nothing. active=1 → banner appears
-- above every page with the stored summary. Ops toggle from /situational
-- without redeploying.

CREATE TABLE IF NOT EXISTS sitrep (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
  summary TEXT NOT NULL DEFAULT '',
  level TEXT NOT NULL DEFAULT 'warn' CHECK (level IN ('warn', 'red')),
  started_at TEXT,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by TEXT
);

INSERT OR IGNORE INTO sitrep (id, active, summary, level)
VALUES (1, 0, '', 'warn');
