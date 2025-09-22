create trigger child_row_trig
  after insert on child referencing new table as new_table
  for each row execute procedure dump_insert();
