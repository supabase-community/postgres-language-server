-- Test adding column without default (should be safe)
-- expect_no_diagnostics
ALTER TABLE users ADD COLUMN email text;