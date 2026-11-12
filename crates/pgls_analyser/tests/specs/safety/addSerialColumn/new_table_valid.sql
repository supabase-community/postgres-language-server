-- Other transactions can't see a table created earlier in the transaction.
-- expect_no_diagnostics
BEGIN;
CREATE TABLE prices (amount int);
ALTER TABLE prices ADD COLUMN id serial;
COMMIT;
