-- expect_lint/addSerialColumn
-- Test adding GENERATED ALWAYS AS ... STORED column to existing table
ALTER TABLE prices ADD COLUMN total integer GENERATED ALWAYS AS (price * quantity) STORED;
