INSERT INTO prt2_adv_p3 SELECT i % 25, i, to_char(i, 'FM0000') FROM generate_series(350, 499) i;
