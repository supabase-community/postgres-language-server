create or replace trigger my_trig
  after insert on parted_trig_1
  for each row execute procedure funcB();
