-- expect_no_diagnostics
CREATE TABLE users (id bigint, email text);
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
