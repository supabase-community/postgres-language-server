create table parted_isvalid_tab_1 partition of parted_isvalid_tab
  for values from (1) to (10) partition by range (a);
