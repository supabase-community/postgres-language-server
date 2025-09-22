INSERT INTO brin_date_test SELECT '5874897-12-01'::date + i FROM generate_series(1, 30) s(i);
