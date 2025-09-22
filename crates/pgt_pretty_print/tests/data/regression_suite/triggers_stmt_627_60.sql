create trigger tg after insert on parent
  for each row execute procedure trig_nothing();
