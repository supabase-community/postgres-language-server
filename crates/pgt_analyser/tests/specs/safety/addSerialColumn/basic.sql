-- expect_lint/safety/addSerialColumn
-- Test adding serial column to existing table
ALTER TABLE prices ADD COLUMN id serial;
