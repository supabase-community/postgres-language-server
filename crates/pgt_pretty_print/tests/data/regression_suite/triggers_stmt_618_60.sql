create trigger tg after insert on parent
  for each row execute function trig_nothing();
