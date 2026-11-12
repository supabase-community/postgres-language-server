-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addingNotNullField
BEGIN;
CREATE TABLE users (id bigint, email text);
COMMIT;
BEGIN;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
COMMIT;
