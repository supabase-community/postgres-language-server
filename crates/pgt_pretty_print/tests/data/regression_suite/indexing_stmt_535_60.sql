create table parted_isvalid_tab_2 partition of parted_isvalid_tab
  for values from (10) to (20) partition by range (a);
