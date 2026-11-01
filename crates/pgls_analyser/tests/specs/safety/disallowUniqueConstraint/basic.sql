-- expect_lint/disallowUniqueConstraint
-- expect_lint/disallowUniqueConstraint
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
ALTER TABLE users ADD COLUMN handle text UNIQUE;
