-- sectionMarkers: goose
-- goose runs each section in a transaction, which an explicit transaction would end.
-- +goose Up
-- expect_lint/transactionNesting
BEGIN;
ALTER TABLE users ADD COLUMN name text;
-- expect_lint/transactionNesting
COMMIT;

-- +goose Down
ALTER TABLE users DROP COLUMN name;
