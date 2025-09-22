ALTER TABLE fk_partitioned_pk ADD CONSTRAINT selffk FOREIGN KEY (a, b) REFERENCES fk_partitioned_pk NOT VALID;
