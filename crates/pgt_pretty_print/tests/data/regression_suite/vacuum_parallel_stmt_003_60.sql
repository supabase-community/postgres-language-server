INSERT INTO parallel_vacuum_table SELECT i from generate_series(1, 10000) i;
