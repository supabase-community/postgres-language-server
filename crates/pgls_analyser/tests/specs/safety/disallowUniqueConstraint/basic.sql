-- expect_lint/disallowUniqueConstraint
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
-- expect_lint/disallowUniqueConstraint
ALTER TABLE users ADD COLUMN handle text UNIQUE;
ALTER TABLE users ADD CONSTRAINT users_name_key UNIQUE USING INDEX users_name_idx;
