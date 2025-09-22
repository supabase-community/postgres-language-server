create trigger tg after insert on child1
  for each row execute function trig_nothing();
