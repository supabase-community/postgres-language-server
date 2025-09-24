-- Test proper robust statements (should be safe)
-- expect_no_diagnostics
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
DROP INDEX CONCURRENTLY IF EXISTS old_idx;