-- sectionMarkers: dbmate
-- The up section runs without a transaction, the down section in one.
-- migrate:up transaction:false
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
CREATE INDEX CONCURRENTLY users_name_idx ON users (name);

-- migrate:down
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
