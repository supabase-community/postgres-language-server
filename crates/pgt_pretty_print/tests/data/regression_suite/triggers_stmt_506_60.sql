create table parted_1 partition of parted for values in (1)
  partition by list (b);
