-- Test single ALTER TABLE statement (should be safe)
-- expect_no_diagnostics
ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
