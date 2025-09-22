create or replace trigger my_trig
  after insert on parted_trig
  for each row execute procedure funcB();
