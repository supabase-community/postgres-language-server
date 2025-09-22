INSERT INTO brin_interval_test SELECT (i || ' days')::interval FROM generate_series(100, 140) s(i);
