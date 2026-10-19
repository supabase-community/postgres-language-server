-- The up section runs without a transaction, the down section in one.
-- section: up transaction:false
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
CREATE INDEX CONCURRENTLY users_name_idx ON users (name);

-- section: down
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
