-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addSerialColumn
BEGIN;
CREATE TABLE prices (amount int);
COMMIT;
BEGIN;
ALTER TABLE prices ADD COLUMN id serial;
COMMIT;
