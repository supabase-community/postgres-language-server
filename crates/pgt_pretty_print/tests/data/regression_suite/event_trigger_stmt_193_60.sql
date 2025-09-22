CREATE TABLE concur_reindex_child PARTITION OF concur_reindex_part
  FOR VALUES FROM (0) TO (10);
