create table trigger_parted_p1 partition of trigger_parted for values in (1)
  partition by list (a);
