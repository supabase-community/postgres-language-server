CREATE TABLE fk_partitioned_fk (b int, a int) PARTITION BY RANGE (a, b);
