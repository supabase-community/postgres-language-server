CREATE TABLE measurements (value int, CONSTRAINT measurements_value_nn NOT NULL value);
ALTER TABLE measurements ADD CONSTRAINT measurements_value_nn2 NOT NULL value NO INHERIT NOT VALID;
ALTER DOMAIN positive_int ADD CONSTRAINT positive_int_nn NOT NULL;
ALTER DOMAIN positive_int ADD NOT NULL;
