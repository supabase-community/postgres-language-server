CREATE TABLE moneyp_12 PARTITION OF moneyp FOR VALUES IN (to_char(12, '99')::int);
