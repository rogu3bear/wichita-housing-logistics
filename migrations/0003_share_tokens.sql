-- Customer-facing case pages share an unguessable token per household.
-- 24 hex chars (12 random bytes) is long enough for a pilot with low
-- sensitivity data and short enough to text to someone on a feature phone.
-- Token is authentication — anyone who has the URL can read that
-- household's case page and post one-way updates back to the activity feed.

ALTER TABLE households ADD COLUMN share_token TEXT;

-- Backfill every existing household with a unique token.
UPDATE households
SET share_token = lower(hex(randomblob(12)))
WHERE share_token IS NULL;

-- Future inserts must supply one (generated SQL-side via lower(hex(randomblob(12)))).
CREATE UNIQUE INDEX IF NOT EXISTS idx_households_share_token
ON households (share_token);
