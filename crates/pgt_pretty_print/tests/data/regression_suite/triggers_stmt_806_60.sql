create trigger table1_trig
  after insert on table1 referencing new table as new_table
  for each statement execute procedure dump_insert();
