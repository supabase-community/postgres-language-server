INSERT INTO brin_interval_test SELECT (i || ' years')::interval FROM generate_series( 177999980,  178000000) s(i);
