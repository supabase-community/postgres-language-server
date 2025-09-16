-- expect_lint/safety/banConcurrentIndexCreationInTransaction
CREATE INDEX CONCURRENTLY "field_name_idx" ON "table_name" ("field_name");
SELECT 1;