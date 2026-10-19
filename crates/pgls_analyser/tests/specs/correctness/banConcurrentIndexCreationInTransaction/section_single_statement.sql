-- The migration tool runs each section in a transaction, even with a single statement.
-- section: up
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);

-- section: down
DROP INDEX users_email_idx;
