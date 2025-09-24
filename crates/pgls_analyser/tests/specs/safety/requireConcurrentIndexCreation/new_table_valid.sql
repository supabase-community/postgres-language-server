-- Test index on newly created table (should be safe)
-- expect_no_diagnostics
CREATE TABLE users (id serial, email text);
CREATE INDEX users_email_idx ON users (email);