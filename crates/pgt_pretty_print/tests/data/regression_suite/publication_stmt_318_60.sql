CREATE TABLE testpub_tbl8_1 PARTITION OF testpub_tbl8 FOR VALUES WITH (modulus 2, remainder 1);
