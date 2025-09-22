create table parted_replica_tab_1 partition of parted_replica_tab
  for values from (1) to (10) partition by range (id);
