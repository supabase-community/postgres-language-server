CREATE TABLE stats_import.part_child_1
  PARTITION OF stats_import.part_parent
  FOR VALUES FROM (0) TO (10)
  WITH (autovacuum_enabled = false);
