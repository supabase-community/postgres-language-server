ALTER TABLE fk_partitioned_fk_full ADD FOREIGN KEY (x, y) REFERENCES fk_notpartitioned_pk MATCH FULL;
