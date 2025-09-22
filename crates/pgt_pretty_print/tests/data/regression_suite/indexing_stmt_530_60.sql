alter table parted_inval_tab attach partition parted_inval_tab_1
  for values from (1) to (100);
