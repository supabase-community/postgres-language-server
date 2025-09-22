CREATE TABLE fk_partitioned_fk_6 (a int REFERENCES fk_partitioned_pk_61) PARTITION BY LIST (a);
