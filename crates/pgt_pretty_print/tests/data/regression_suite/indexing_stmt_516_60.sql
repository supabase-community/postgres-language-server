create table parted_index_col_drop1 partition of parted_index_col_drop
  for values in (1) partition by list (a);
