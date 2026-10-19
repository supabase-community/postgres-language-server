-- sectionMarkers: dbmate
-- migrate:up transaction:false
BEGIN;
-- expect_lint/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
COMMIT;

-- migrate:down transaction:false
DROP INDEX CONCURRENTLY users_email_idx;
