INSERT INTO plt1_adv SELECT i, i, to_char(i % 5, 'FM0000') FROM generate_series(0, 24) i;
