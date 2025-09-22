CREATE TABLE fk_partitioned_pk (a int, b int, PRIMARY KEY (a, b)) PARTITION BY RANGE (a, b);
