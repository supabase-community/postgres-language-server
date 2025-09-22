INSERT INTO brin_date_test SELECT '4713-01-01 BC'::date + i FROM generate_series(1, 30) s(i);
