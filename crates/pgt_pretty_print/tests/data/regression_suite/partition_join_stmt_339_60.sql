INSERT INTO prt3_adv SELECT i, i % 25, to_char(i, 'FM0000') FROM generate_series(200, 399) i;
