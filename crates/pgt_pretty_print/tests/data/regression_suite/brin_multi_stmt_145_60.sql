INSERT INTO brin_date_test SELECT '2000-01-01'::date + i FROM generate_series(1, 40) s(i);
