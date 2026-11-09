-- A NOT NULL constraint (Postgres 18) without NOT VALID scans the table like SET NOT NULL
-- expect_lint/addingNotNullField
ALTER TABLE users ADD CONSTRAINT users_email_not_null NOT NULL email;
-- expect_lint/addingNotNullField
ALTER TABLE users ADD NOT NULL name;
ALTER TABLE users ADD CONSTRAINT users_age_not_null NOT NULL age NOT VALID;
ALTER TABLE users VALIDATE CONSTRAINT users_age_not_null;
