create trigger trans_tab_parent_delete_trig
  after delete on trans_tab_parent referencing old table as old_table
  for each statement execute procedure dump_delete();
