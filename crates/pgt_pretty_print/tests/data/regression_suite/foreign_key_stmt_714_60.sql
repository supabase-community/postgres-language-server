CREATE TABLE fk_partitioned_fk_4_2 (a int, b int, FOREIGN KEY (a, b) REFERENCES fk_notpartitioned_pk(a, b) ON UPDATE SET NULL);
