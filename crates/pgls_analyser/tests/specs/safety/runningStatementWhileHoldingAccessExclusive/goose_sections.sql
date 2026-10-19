-- sectionMarkers: goose
-- +goose Up
ALTER TABLE authors ADD COLUMN email text;
-- expect_lint/runningStatementWhileHoldingAccessExclusive
UPDATE authors SET email = '';

-- +goose Down
ALTER TABLE authors DROP COLUMN email;
