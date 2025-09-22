create trigger make_bogus_matview
  after insert on my_table
  referencing new table as new_table
  for each statement execute function make_bogus_matview();
