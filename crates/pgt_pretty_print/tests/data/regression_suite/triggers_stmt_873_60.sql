create trigger merge_target_table_delete_trig
  after delete on merge_target_table referencing old table as old_table
  for each statement execute procedure dump_delete();
