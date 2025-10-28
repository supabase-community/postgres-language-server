-- Test adding regular column with integer type (should be safe)
-- expect_no_diagnostics
ALTER TABLE prices ADD COLUMN quantity integer DEFAULT 0;
