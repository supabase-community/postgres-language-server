-- Test single ALTER TABLE with multiple actions (should be safe)
-- expect_no_diagnostics
ALTER TABLE authors
  ALTER COLUMN name SET NOT NULL,
  ALTER COLUMN email SET NOT NULL;
