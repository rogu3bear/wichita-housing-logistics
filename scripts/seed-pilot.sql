-- Pilot-demo seed for the remote database.
--
-- Populates households/resources/placements/activity_notes with a coherent
-- week-and-a-half of fictional but realistic operations data so the app
-- looks and reads like it's been in active use. All names, phones, and
-- addresses are fictional; phones use 316 (Wichita area code) with the
-- 555-01xx block reserved for fiction.
--
-- Safe to re-run: truncates all four pilot-relevant tables first and
-- resets sqlite_sequence so IDs restart at 1. This is destructive — only
-- run against an environment where wiping demo data is intended.
--
-- Apply via:
--   set -a; . .cf-tokens.local/whl-d1-migrate.env; set +a
--   bunx wrangler d1 execute wichita-housing-logistics-db --remote \
--     --file scripts/seed-pilot.sql
--
-- D1 wraps the whole file in its own atomic execution, so no BEGIN/COMMIT
-- — those are rejected by the runtime ("use state.storage.transaction()").

DELETE FROM activity_notes;
DELETE FROM placements;
DELETE FROM housing_resources;
DELETE FROM households;
DELETE FROM sqlite_sequence
  WHERE name IN ('activity_notes','placements','housing_resources','households');

-- Households — distributed across every stage so dashboard counters
-- render non-zero values everywhere.
INSERT INTO households
  (id, head_name, household_size, phone, email, stage, intake_notes, share_token, created_at, updated_at)
VALUES
  (1, 'Soto, Brian',        1, '316-555-0147', NULL,
      'intake',
      'Self-referral via 2-1-1. Slept in car past three nights after losing rooming house in Delano. No income, interested in VASH eligibility screen.',
      lower(hex(randomblob(12))),
      datetime('now','-2 days'), datetime('now','-2 days')),

  (2, 'Chen, Lian',         5, '316-555-0123', 'lchen.pilot@example.org',
      'intake',
      'Family of 5 in motel voucher expiring Friday. Two kids in WUSD 259. Spouse works nights at Spirit. Needs 3BR minimum.',
      lower(hex(randomblob(12))),
      datetime('now','-3 days'), datetime('now','-1 days')),

  (3, 'Alvarez, Maria',     3, '316-555-0141', 'malvarez.pilot@example.org',
      'assessment',
      'Single mother, two school-age kids (6, 9). Working part-time at HCA, unstable hours. Left a doubled-up arrangement after landlord posted 30-day notice.',
      lower(hex(randomblob(12))),
      datetime('now','-12 days'), datetime('now','-4 days')),

  (4, 'Becker, Daniel',     1, '316-555-0172', NULL,
      'assessment',
      'Veteran, Army 03-07. VA Wichita referral. SSVF eligible. Requested male case manager. ROI signed 2026-04-10.',
      lower(hex(randomblob(12))),
      datetime('now','-10 days'), datetime('now','-5 days')),

  (5, 'Williams, Jada',     2, '316-555-0188', NULL,
      'assessment',
      'DV survivor, left partner 2026-04-06. One child, age 4. Safety plan active; do not share address with inquirers. StepStone coordinating.',
      lower(hex(randomblob(12))),
      datetime('now','-11 days'), datetime('now','-2 days')),

  (6, 'Okonkwo, Priya',     4, '316-555-0118', 'priyaok.pilot@example.org',
      'placement',
      'Family of four, needs 2+ BR. Approved for Sunflower Transitional; awaiting signed lease addendum from property manager.',
      lower(hex(randomblob(12))),
      datetime('now','-18 days'), datetime('now','-1 days')),

  (7, 'Navarro, Elena',     1, '316-555-0153', NULL,
      'placement',
      'Senior, 68, fixed income (SSDI $1,140). Mobility impairment — ground floor required. Matched to Cedar Crossing 112 (ADA).',
      lower(hex(randomblob(12))),
      datetime('now','-16 days'), datetime('now','-5 days')),

  (8, 'Hollis, Trey',       1, '316-555-0196', 'trey.h.pilot@example.org',
      'placement',
      'Aging out of foster care 2026-04-30. Has TANF application pending, no rental history. Independent Living coordinator copied on placement plan.',
      lower(hex(randomblob(12))),
      datetime('now','-14 days'), datetime('now','-8 days')),

  (9, 'Greer, Marcus',      2, '316-555-0167', 'mgreer.pilot@example.org',
      'follow_up',
      'Moved in 2026-04-04. Working at Cargill. Needs furniture support (Catholic Charities voucher submitted). 30-day stability check coming up.',
      lower(hex(randomblob(12))),
      datetime('now','-22 days'), datetime('now','-6 days')),

  (10,'Torres, Camila',     2, '316-555-0109', NULL,
      'exited',
      'Self-resolved exit 2026-04-12. Reunited with sister in Oklahoma City. Left shelter bed, case closed at her request.',
      lower(hex(randomblob(12))),
      datetime('now','-24 days'), datetime('now','-6 days'));

-- Housing resources — mix of kind and status so the inventory page
-- shows the full palette.
INSERT INTO housing_resources
  (id, label, kind, address, capacity, status, notes, created_at, updated_at)
VALUES
  (1, 'HopeNet Shelter Bed 12',       'shelter_bed',
      '1324 N Broadway, Wichita, KS 67214', 1, 'available',
      'Low-barrier bed. Walk-ins 6pm-9pm nightly.',
      datetime('now','-60 days'), datetime('now','-6 days')),

  (2, 'HopeNet Shelter Bed 7',        'shelter_bed',
      '1324 N Broadway, Wichita, KS 67214', 1, 'occupied',
      'Assigned to Soto on intake bridge; 7-day cap before transition plan.',
      datetime('now','-60 days'), datetime('now','-2 days')),

  (3, 'Sunflower Transitional Apt 2B','transitional',
      '915 S Market, Wichita, KS 67211', 3, 'held',
      'Hold for Okonkwo pending landlord signature on addendum.',
      datetime('now','-45 days'), datetime('now','-1 days')),

  (4, 'Sunflower Transitional Apt 3A','transitional',
      '915 S Market, Wichita, KS 67211', 2, 'occupied',
      'Youth placement (Hollis). IL coordinator has key access.',
      datetime('now','-45 days'), datetime('now','-8 days')),

  (5, 'Cedar Crossing Unit 204',      'permanent_supportive',
      '2201 E Central, Wichita, KS 67214', 5, 'available',
      '2BR, ADA accessible. Reserve for families w/ mobility needs.',
      datetime('now','-90 days'), datetime('now','-30 days')),

  (6, 'Cedar Crossing Unit 112',      'permanent_supportive',
      '2201 E Central, Wichita, KS 67214', 1, 'occupied',
      'Ground floor ADA. Navarro moved in 2026-04-13.',
      datetime('now','-90 days'), datetime('now','-5 days')),

  (7, 'Eastside Rental 3B',           'rental_unit',
      '3408 E Harry, Wichita, KS 67218', 4, 'occupied',
      '2BR/1BA. Greer household, lease starts 2026-04-04.',
      datetime('now','-75 days'), datetime('now','-14 days')),

  (8, 'North Riverside Efficiency',   'rental_unit',
      '621 N Waco, Wichita, KS 67203', 1, 'offline',
      'Plumbing repair scheduled 2026-04-22. Back online ~1 week.',
      datetime('now','-75 days'), datetime('now','-3 days'));

-- Placement lifecycle: every stage except 'proposed' is represented, since
-- proposed is the empty state before the confirm step. Timestamps are
-- relative so the "X days ago" reads cleanly in the UI.
INSERT INTO placements
  (id, household_id, resource_id, status, started_at, ended_at, notes, created_at, updated_at)
VALUES
  (1, 6, 3, 'confirmed',  NULL,                          NULL,
      'Hold confirmed; awaiting landlord addendum.',
      datetime('now','-5 days'),  datetime('now','-1 days')),

  (2, 7, 6, 'moved_in',   datetime('now','-5 days'),     NULL,
      'Keys delivered by coord_singh. ADA check complete.',
      datetime('now','-9 days'),  datetime('now','-5 days')),

  (3, 8, 4, 'moved_in',   datetime('now','-8 days'),     NULL,
      'IL handoff complete. 14-day check scheduled.',
      datetime('now','-12 days'), datetime('now','-8 days')),

  (4, 9, 7, 'moved_in',   datetime('now','-14 days'),    NULL,
      'Lease signed; furniture voucher pending.',
      datetime('now','-18 days'), datetime('now','-14 days')),

  (5,10, 2, 'exited',     datetime('now','-16 days'),    datetime('now','-6 days'),
      'Household self-exited to family in OKC. Bed released.',
      datetime('now','-20 days'), datetime('now','-6 days'));

-- Activity feed — ordered oldest → newest so the UI's most-recent-first
-- query surfaces the late-stage events. Mix of case_manager / coord /
-- household authors gives the feed a natural cadence.
INSERT INTO activity_notes (id, entity_type, entity_id, author, body, created_at)
VALUES
  ( 1, 'household',  10, 'case_manager_kim', 'Intake complete. HopeNet bed 7 assigned same night.',         datetime('now','-24 days')),
  ( 2, 'household',   9, 'case_manager_lee', 'Assessment complete. Matched to Eastside Rental 3B.',          datetime('now','-20 days')),
  ( 3, 'placement',   4, 'case_manager_lee', 'Lease signed; move-in coordinated for 2026-04-04.',            datetime('now','-18 days')),
  ( 4, 'household',   6, 'case_manager_kim', 'Intake call complete. Family of four, needs 2BR.',             datetime('now','-18 days')),
  ( 5, 'household',   8, 'case_manager_rivera', 'Foster-care aging-out referral received from DCF.',         datetime('now','-14 days')),
  ( 6, 'placement',   4, 'household',         'Got the keys, thanks! Will call back after we unpack.',      datetime('now','-14 days')),
  ( 7, 'household',   7, 'coord_singh',       'Matched to Cedar Crossing 112 (ADA, ground floor).',          datetime('now','-12 days')),
  ( 8, 'household',   3, 'case_manager_kim',  'Scheduled assessment 2026-04-12.',                           datetime('now','-12 days')),
  ( 9, 'placement',   3, 'case_manager_rivera', 'Sunflower 3A offered to Hollis pending IL signoff.',       datetime('now','-12 days')),
  (10, 'household',   5, 'case_manager_kim',  'DV intake. StepStone coordinating safety plan.',              datetime('now','-11 days')),
  (11, 'household',   4, 'case_manager_lee',  'VA Wichita confirmed SSVF eligibility.',                      datetime('now','-10 days')),
  (12, 'placement',   3, 'coord_singh',       'Keys delivered to Hollis. IL coordinator on site.',           datetime('now','-8 days')),
  (13, 'household',   8, 'case_manager_rivera','Moved in at Sunflower 3A. 14-day check scheduled.',          datetime('now','-8 days')),
  (14, 'household',   9, 'household',         'Furniture voucher status update? Kids starting school Mon.',  datetime('now','-7 days')),
  (15, 'household',   9, 'case_manager_lee',  'Furniture voucher approved. Catholic Charities delivery Tue.', datetime('now','-6 days')),
  (16, 'placement',   5, 'case_manager_kim',  'Torres exited to family in OKC. Bed released.',              datetime('now','-6 days')),
  (17, 'resource',    3, 'coord_singh',       'Hold extended 7 days pending landlord addendum.',             datetime('now','-5 days')),
  (18, 'placement',   2, 'coord_singh',       'Navarro keys delivered. ADA check complete.',                 datetime('now','-5 days')),
  (19, 'household',   2, 'case_manager_kim',  'Motel voucher expires Friday. Need bridge placement.',        datetime('now','-3 days')),
  (20, 'resource',    8, 'coord_singh',       'Offline for plumbing repair through ~2026-04-22.',           datetime('now','-3 days')),
  (21, 'household',   1, 'case_manager_kim',  'Walked in this afternoon. HopeNet 7 assigned for tonight.',   datetime('now','-2 days')),
  (22, 'household',   5, 'household',         'Thank you for the safety plan help. Feeling steadier.',      datetime('now','-2 days')),
  (23, 'household',   6, 'case_manager_kim',  'Still waiting on addendum. Called landlord again today.',    datetime('now','-1 days')),
  (24, 'household',   2, 'household',         'Spouse working nights at Spirit — any placement w/ late entry?', datetime('now','-1 days')),
  (25, 'system',   NULL, 'system',            'Pilot demo data loaded.',                                     datetime('now'));
