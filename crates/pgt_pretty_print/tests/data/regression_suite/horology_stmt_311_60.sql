SELECT i, to_timestamp('20181102123456123456', 'YYYYMMDDHH24MISSFF' || i) FROM generate_series(1, 6) i;
