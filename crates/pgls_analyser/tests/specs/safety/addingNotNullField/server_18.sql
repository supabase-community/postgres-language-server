-- On Postgres 18, the note suggests a NOT VALID NOT NULL constraint instead of a CHECK constraint
-- server_version: 18
-- expect_lint/addingNotNullField
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
