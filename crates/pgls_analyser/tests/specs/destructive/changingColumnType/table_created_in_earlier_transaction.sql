-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/changingColumnType
BEGIN;
CREATE TABLE users (id int, name varchar(100));
COMMIT;
BEGIN;
ALTER TABLE users ALTER COLUMN id TYPE bigint;
COMMIT;
