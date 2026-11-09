-- The search path changes between the statements, so they alter different tables.
-- expect_no_diagnostics
ALTER TABLE authors ADD COLUMN name text;
SET search_path = app;
ALTER TABLE authors ADD COLUMN email text;
