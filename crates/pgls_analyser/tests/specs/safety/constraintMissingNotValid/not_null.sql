-- A NOT NULL constraint (Postgres 18) can be added as NOT VALID too
-- expect_lint/constraintMissingNotValid
ALTER TABLE users ADD CONSTRAINT users_email_not_null NOT NULL email;
-- expect_lint/constraintMissingNotValid
ALTER TABLE users ADD NOT NULL name;
ALTER TABLE users ADD CONSTRAINT users_age_not_null NOT NULL age NOT VALID;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
