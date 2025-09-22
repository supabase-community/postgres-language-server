create trigger my_trig
  after insert on my_table
  for each row execute procedure funcA();
