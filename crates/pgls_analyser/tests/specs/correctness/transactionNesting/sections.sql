-- The migration tool runs each section in a transaction, which an explicit transaction would end.
-- section: up
-- expect_lint/transactionNesting
BEGIN;
ALTER TABLE users ADD COLUMN name text;
-- expect_lint/transactionNesting
COMMIT;

-- section: down
ALTER TABLE users DROP COLUMN name;
