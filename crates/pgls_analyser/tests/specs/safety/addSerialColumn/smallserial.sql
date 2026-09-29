-- expect_lint/addSerialColumn
-- Test adding smallserial column to existing table
ALTER TABLE prices ADD COLUMN small_id smallserial;
