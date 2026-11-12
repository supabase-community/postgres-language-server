-- Other transactions can't see a table created earlier in the transaction.
-- expect_no_diagnostics
BEGIN;
CREATE TABLE bookings (room int, during tsrange);
ALTER TABLE bookings ADD CONSTRAINT bookings_no_overlap EXCLUDE USING gist (room WITH =, during WITH &&);
COMMIT;
