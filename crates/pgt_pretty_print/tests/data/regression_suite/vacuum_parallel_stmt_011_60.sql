INSERT INTO parallel_vacuum_table SELECT i FROM generate_series(1, 10000) i;
