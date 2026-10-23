-- The up section runs without a transaction, the down section in one.
-- section: up transaction:false
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);

-- section: down
-- expect_lint/banConcurrentIndexCreationInTransaction
DROP INDEX CONCURRENTLY users_email_idx;
