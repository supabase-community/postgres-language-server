create rule base_tab_def_view_ins_rule as on insert to base_tab_def_view
  do also insert into base_tab_def values (new.a, new.b, new.c, new.d, new.e);
