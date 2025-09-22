create trigger trans_tab_parent_insert_trig
  after insert on trans_tab_parent referencing new table as new_table
  for each statement execute procedure dump_insert();
