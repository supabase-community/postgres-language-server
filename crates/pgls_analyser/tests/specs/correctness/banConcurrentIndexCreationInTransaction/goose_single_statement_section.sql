-- sectionMarkers: goose
-- goose runs each section in a transaction, even with a single statement.
-- +goose Up
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);

-- +goose Down
DROP INDEX users_email_idx;
