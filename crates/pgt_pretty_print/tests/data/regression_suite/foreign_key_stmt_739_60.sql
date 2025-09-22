CREATE TABLE fk_partitioned_fk_6 (a int,
	FOREIGN KEY (a) REFERENCES fk_partitioned_pk_6,
	FOREIGN KEY (a) REFERENCES fk_partitioned_pk_6
) PARTITION BY LIST (a);
