-- Postgres runs none of these in a transaction block.
-- expect_lint/banConcurrentIndexCreationInTransaction
DROP INDEX CONCURRENTLY users_email_idx;
-- expect_lint/banConcurrentIndexCreationInTransaction
REINDEX INDEX CONCURRENTLY users_name_idx;
-- expect_lint/banConcurrentIndexCreationInTransaction
REINDEX (CONCURRENTLY, VERBOSE) TABLE users;
-- expect_lint/banConcurrentIndexCreationInTransaction
ALTER TABLE events DETACH PARTITION events_2024 CONCURRENTLY;
REINDEX (CONCURRENTLY false) TABLE users;
REFRESH MATERIALIZED VIEW CONCURRENTLY user_stats;
