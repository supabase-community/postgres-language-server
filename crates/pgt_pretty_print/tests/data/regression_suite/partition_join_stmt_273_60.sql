INSERT INTO prt2_adv_p2 SELECT i % 25, i, to_char(i, 'FM0000') FROM generate_series(200, 299) i;
