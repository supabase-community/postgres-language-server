-- sectionMarkers: dbmate
-- Without a transaction from dbmate, explicit transactions are fine.
-- migrate:up transaction:false
BEGIN;
ALTER TABLE users ADD COLUMN name text;
COMMIT;
CREATE INDEX CONCURRENTLY users_name_idx ON users (name);
BEGIN;
-- expect_lint/transactionNesting
BEGIN;
COMMIT;

-- migrate:down transaction:false
DROP INDEX CONCURRENTLY users_name_idx;
-- expect_lint/transactionNesting
COMMIT;
