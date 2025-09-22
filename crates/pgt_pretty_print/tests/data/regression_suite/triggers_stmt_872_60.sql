create trigger merge_target_table_update_trig
  after update on merge_target_table referencing old table as old_table new table as new_table
  for each statement execute procedure dump_update();
