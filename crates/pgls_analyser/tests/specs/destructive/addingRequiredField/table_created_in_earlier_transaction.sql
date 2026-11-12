-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addingRequiredField
BEGIN;
CREATE TABLE users (id bigint);
COMMIT;
BEGIN;
ALTER TABLE users ADD COLUMN email text NOT NULL;
COMMIT;
