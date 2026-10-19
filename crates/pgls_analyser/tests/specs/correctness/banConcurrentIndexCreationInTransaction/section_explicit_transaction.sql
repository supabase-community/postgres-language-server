-- section: up transaction:false
BEGIN;
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
COMMIT;

-- section: down transaction:false
DROP INDEX CONCURRENTLY users_email_idx;
