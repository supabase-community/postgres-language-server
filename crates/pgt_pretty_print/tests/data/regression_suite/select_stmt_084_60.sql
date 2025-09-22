create table list_parted_tbl1 partition of list_parted_tbl
  for values in (1) partition by list(b);
