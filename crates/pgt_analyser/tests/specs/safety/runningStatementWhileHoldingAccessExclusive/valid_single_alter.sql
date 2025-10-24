-- Valid: ALTER TABLE alone in transaction (no subsequent statements)
-- expect_no_diagnostics
ALTER TABLE authors ADD COLUMN email TEXT;
