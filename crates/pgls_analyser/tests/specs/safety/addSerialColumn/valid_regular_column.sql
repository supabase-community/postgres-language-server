-- Test adding regular column (should be safe)
-- expect_no_diagnostics
ALTER TABLE prices ADD COLUMN name text;
