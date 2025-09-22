create trigger trans_tab_parent_update_trig
  after update on trans_tab_parent referencing old table as old_table
  for each statement execute procedure dump_update_old();
