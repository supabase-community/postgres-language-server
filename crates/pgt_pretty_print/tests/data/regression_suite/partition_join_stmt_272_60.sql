INSERT INTO prt2_adv_p1 SELECT i % 25, i, to_char(i, 'FM0000') FROM generate_series(100, 149) i;
