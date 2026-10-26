-- dbmate sends a section to Postgres as one query, which runs in a transaction block if it has
-- more than one statement, also after a COMMIT.
-- section: up transaction:false
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
BEGIN;
COMMIT;
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_name_idx ON users (name);
