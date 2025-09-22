INSERT INTO alpha_pos SELECT  1.0, i, to_char(i % 10, 'FM0000') FROM generate_series(100, 399) i WHERE i % 10 IN (1, 3, 4, 6, 8, 9);
