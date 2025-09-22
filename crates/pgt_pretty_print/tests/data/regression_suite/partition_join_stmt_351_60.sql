INSERT INTO prt1_adv SELECT i, i % 25, to_char(i, 'FM0000') FROM generate_series(100, 399) i;
