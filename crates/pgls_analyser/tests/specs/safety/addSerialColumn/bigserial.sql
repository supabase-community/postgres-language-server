-- expect_lint/addSerialColumn
-- Test adding bigserial column to existing table
ALTER TABLE prices ADD COLUMN big_id bigserial;
