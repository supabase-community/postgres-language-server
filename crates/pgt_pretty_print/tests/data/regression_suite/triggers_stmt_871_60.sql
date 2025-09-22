create trigger merge_target_table_insert_trig
  after insert on merge_target_table referencing new table as new_table
  for each statement execute procedure dump_insert();
