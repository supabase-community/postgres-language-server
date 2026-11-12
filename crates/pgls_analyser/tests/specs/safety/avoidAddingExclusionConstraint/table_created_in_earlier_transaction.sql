-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/avoidAddingExclusionConstraint
BEGIN;
CREATE TABLE bookings (room int, during tsrange);
COMMIT;
BEGIN;
ALTER TABLE bookings ADD CONSTRAINT bookings_no_overlap EXCLUDE USING gist (room WITH =, during WITH &&);
COMMIT;
