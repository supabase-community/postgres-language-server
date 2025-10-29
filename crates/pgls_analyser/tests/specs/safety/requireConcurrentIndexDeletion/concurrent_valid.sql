-- Test concurrent index deletion (should be safe)
-- expect_no_diagnostics
DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;