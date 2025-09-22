INSERT INTO prt2_adv SELECT i % 25, i, to_char(i, 'FM0000') FROM generate_series(500, 599) i;
