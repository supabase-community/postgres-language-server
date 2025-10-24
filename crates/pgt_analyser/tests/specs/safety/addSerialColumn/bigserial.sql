-- expect_lint/safety/addSerialColumn
-- Test adding bigserial column to existing table
ALTER TABLE prices ADD COLUMN big_id bigserial;
