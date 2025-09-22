CREATE TABLE fk_partitioned_fk_2 (b int, c text, a int,
	FOREIGN KEY (a, b) REFERENCES fk_notpartitioned_pk ON UPDATE CASCADE ON DELETE CASCADE);
