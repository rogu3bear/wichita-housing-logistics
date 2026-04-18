-- Households tracked through the logistics pipeline.
CREATE TABLE IF NOT EXISTS households (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  head_name TEXT NOT NULL,
  household_size INTEGER NOT NULL DEFAULT 1 CHECK (household_size > 0),
  phone TEXT,
  email TEXT,
  stage TEXT NOT NULL DEFAULT 'intake'
    CHECK (stage IN ('intake','assessment','placement','follow_up','exited')),
  intake_notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_households_stage_id
ON households (stage, id DESC);

-- Housing inventory (beds, units, placements stock).
CREATE TABLE IF NOT EXISTS housing_resources (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  label TEXT NOT NULL,
  kind TEXT NOT NULL
    CHECK (kind IN ('shelter_bed','transitional','permanent_supportive','rental_unit','other')),
  address TEXT,
  capacity INTEGER NOT NULL DEFAULT 1 CHECK (capacity > 0),
  status TEXT NOT NULL DEFAULT 'available'
    CHECK (status IN ('available','held','occupied','offline')),
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_resources_status_id
ON housing_resources (status, id DESC);

-- Placement lifecycle: proposed → confirmed → moved_in → exited (or cancelled).
CREATE TABLE IF NOT EXISTS placements (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  household_id INTEGER NOT NULL REFERENCES households(id),
  resource_id INTEGER NOT NULL REFERENCES housing_resources(id),
  status TEXT NOT NULL DEFAULT 'proposed'
    CHECK (status IN ('proposed','confirmed','moved_in','exited','cancelled')),
  started_at TEXT,
  ended_at TEXT,
  notes TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_placements_status_id
ON placements (status, id DESC);

CREATE INDEX IF NOT EXISTS idx_placements_household_id
ON placements (household_id, id DESC);

CREATE INDEX IF NOT EXISTS idx_placements_resource_id
ON placements (resource_id, id DESC);

-- Activity audit trail. entity_id is nullable for 'system' entries.
CREATE TABLE IF NOT EXISTS activity_notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  entity_type TEXT NOT NULL
    CHECK (entity_type IN ('household','resource','placement','system')),
  entity_id INTEGER,
  author TEXT NOT NULL DEFAULT 'system',
  body TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_activity_created
ON activity_notes (id DESC);

CREATE INDEX IF NOT EXISTS idx_activity_entity
ON activity_notes (entity_type, entity_id, id DESC);
