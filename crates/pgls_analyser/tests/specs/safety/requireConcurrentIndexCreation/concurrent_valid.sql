-- Test concurrent index creation (should be safe)
-- expect_no_diagnostics
CREATE INDEX CONCURRENTLY users_email_idx ON users (email);