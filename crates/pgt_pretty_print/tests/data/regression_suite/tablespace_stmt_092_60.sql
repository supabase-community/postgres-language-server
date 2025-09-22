CREATE TABLE testschema.part_910 PARTITION OF testschema.part FOR VALUES IN (9, 10)
  PARTITION BY LIST (a) TABLESPACE regress_tblspace;
