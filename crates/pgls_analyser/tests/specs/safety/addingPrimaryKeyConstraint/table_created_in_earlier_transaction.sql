-- The table was created in a transaction that committed, so other transactions can see it.
-- expect_lint/addingPrimaryKeyConstraint
BEGIN;
CREATE TABLE users (id bigint, email text);
COMMIT;
BEGIN;
ALTER TABLE users ADD PRIMARY KEY (id);
COMMIT;
