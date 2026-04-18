-- Seed fixtures so the UI is reviewable immediately after a fresh migration.
-- Safe to re-run: every insert guards on a unique identifier already present.

INSERT OR IGNORE INTO households (id, head_name, household_size, phone, stage, intake_notes)
VALUES
  (1, 'Alvarez, Maria', 3, '316-555-0141', 'assessment', 'Single mother, two school-age kids. Working part-time.'),
  (2, 'Becker, Daniel',  1, '316-555-0172', 'intake',     'Veteran, referred by VA Wichita.'),
  (3, 'Okonkwo, Priya',  4, '316-555-0118', 'placement',  'Family of four, needs 2+ bedroom unit.');

INSERT OR IGNORE INTO housing_resources (id, label, kind, address, capacity, status, notes)
VALUES
  (1, 'HopeNet Shelter Bed 12', 'shelter_bed',           '1324 N Broadway, Wichita, KS', 1, 'available', NULL),
  (2, 'Sunflower Transitional Apt 2B', 'transitional',   '915 S Market, Wichita, KS',    3, 'held',      'Hold for Alvarez household'),
  (3, 'Cedar Crossing Unit 204', 'permanent_supportive', '2201 E Central, Wichita, KS',  5, 'available', '2BR, ADA accessible'),
  (4, 'Eastside Rental 3B', 'rental_unit',                '3408 E Harry, Wichita, KS',    4, 'occupied',  NULL);

INSERT OR IGNORE INTO placements (id, household_id, resource_id, status, started_at, notes)
VALUES
  (1, 1, 2, 'confirmed', NULL,                  'Hold confirmed pending move-in date.'),
  (2, 3, 3, 'moved_in',  '2026-04-10 14:00 UTC','Family moved in last week; follow-up scheduled.');

INSERT OR IGNORE INTO activity_notes (id, entity_type, entity_id, author, body)
VALUES
  (1, 'household',  1, 'case_manager_kim', 'Initial intake call completed. Scheduled assessment.'),
  (2, 'household',  2, 'case_manager_lee', 'Received VA referral packet; requesting release of records.'),
  (3, 'resource',   2, 'coord_singh',      'Placed hold for Alvarez until 2026-04-22.'),
  (4, 'placement',  2, 'case_manager_kim', 'Move-in verified; keys delivered.'),
  (5, 'system',     NULL,'system',         'Seed data loaded.');
