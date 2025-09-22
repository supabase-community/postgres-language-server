INSERT INTO brin_interval_test SELECT (i || ' years')::interval FROM generate_series(-178000000, -177999980) s(i);
