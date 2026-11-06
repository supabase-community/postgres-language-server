-- The table is created after the index, so the index is on an existing table.
-- expect_lint/requireConcurrentIndexCreation
CREATE INDEX users_email_idx ON users (email);
CREATE TABLE users (id bigint, email text);
