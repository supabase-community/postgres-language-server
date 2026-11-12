-- The table is created after the constraint, so the constraint is added to an existing table.
-- expect_lint/disallowUniqueConstraint
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
CREATE TABLE users (id bigint, email text);
