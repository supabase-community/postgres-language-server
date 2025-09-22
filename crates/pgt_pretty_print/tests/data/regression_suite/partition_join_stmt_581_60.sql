INSERT INTO beta_neg SELECT -1.0, i, to_char(i % 10, 'FM0000') FROM generate_series(350, 499) i WHERE i % 10 IN (2, 3, 4, 6, 7, 9);
