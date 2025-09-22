CREATE TABLE fk_partitioned_fk_4 (a int, b int, FOREIGN KEY (a, b) REFERENCES fk_notpartitioned_pk(a, b) ON UPDATE CASCADE ON DELETE CASCADE) PARTITION BY RANGE (b, a);
