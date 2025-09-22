CREATE TABLE fk_partitioned_fk_6 (a int REFERENCES fk_partitioned_pk_6) PARTITION BY LIST (a);
