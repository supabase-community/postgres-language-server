CREATE TABLE fk_partitioned_pk_2 PARTITION OF fk_partitioned_pk FOR VALUES FROM (1000,1000) TO (2000,2000);
