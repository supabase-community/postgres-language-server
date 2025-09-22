ALTER TABLE fk_notpartitioned_fk ADD CONSTRAINT fk_notpartitioned_fk_a_b_fkey
	FOREIGN KEY (a, b) REFERENCES fk_partitioned_pk NOT VALID;
