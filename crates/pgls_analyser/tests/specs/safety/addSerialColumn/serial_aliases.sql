-- serial2, serial4 and serial8 are serial types too.
-- expect_lint/addSerialColumn
-- expect_lint/addSerialColumn
-- expect_lint/addSerialColumn
ALTER TABLE prices ADD COLUMN a serial2, ADD COLUMN b serial4, ADD COLUMN c serial8;
