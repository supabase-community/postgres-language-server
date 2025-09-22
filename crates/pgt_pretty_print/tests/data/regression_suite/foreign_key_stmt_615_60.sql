CREATE TABLE fk_partitioned_pk_1 PARTITION OF fk_partitioned_pk FOR VALUES FROM (0,0) TO (1000,1000);
