create table trigger_parted_p2 partition of trigger_parted for values in (2)
  partition by list (a);
