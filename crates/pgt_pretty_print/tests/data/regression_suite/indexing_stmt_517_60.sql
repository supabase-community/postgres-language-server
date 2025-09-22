create table parted_index_col_drop2 partition of parted_index_col_drop
  for values in (2) partition by list (a);
