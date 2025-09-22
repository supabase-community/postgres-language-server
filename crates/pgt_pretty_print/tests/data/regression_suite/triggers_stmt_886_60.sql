create or replace trigger my_trig
  before insert on my_table
  for each row execute procedure funcB();
