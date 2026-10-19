-- Without a transaction from the migration tool, explicit transactions are fine.
-- section: up transaction:false
BEGIN;
ALTER TABLE users ADD COLUMN name text;
COMMIT;
CREATE INDEX CONCURRENTLY users_name_idx ON users (name);
BEGIN;
-- expect_lint/transactionNesting
BEGIN;
COMMIT;

-- section: down transaction:false
DROP INDEX CONCURRENTLY users_name_idx;
-- expect_lint/transactionNesting
COMMIT;
