-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addingFieldWithDefault
BEGIN;
CREATE TABLE users (id bigint);
COMMIT;
BEGIN;
ALTER TABLE users ADD COLUMN token uuid DEFAULT gen_random_uuid();
COMMIT;
