-- expect_lint/safety/multipleAlterTable
-- Test multiple ALTER TABLE statements on the same table
ALTER TABLE authors ALTER COLUMN name SET NOT NULL;
ALTER TABLE authors ALTER COLUMN email SET NOT NULL;
