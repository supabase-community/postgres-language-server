-- Unqualified tables are in the first schema of the search path.
-- expect_lint/multipleAlterTable
SET search_path = app;
ALTER TABLE authors ADD COLUMN name text;
ALTER TABLE app.authors ADD COLUMN email text;
