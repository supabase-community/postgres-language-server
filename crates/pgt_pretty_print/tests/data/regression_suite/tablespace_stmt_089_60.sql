CREATE TABLE testschema.part_56 PARTITION OF testschema.part FOR VALUES IN (5, 6)
  PARTITION BY LIST (a);
