-- expect_lint/addSerialColumn
-- Test adding serial column to existing table
ALTER TABLE prices ADD COLUMN id serial;
