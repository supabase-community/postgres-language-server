SELECT relid, parentrelid, level, isleaf
  FROM pg_partition_tree('ptif_normal_table');
