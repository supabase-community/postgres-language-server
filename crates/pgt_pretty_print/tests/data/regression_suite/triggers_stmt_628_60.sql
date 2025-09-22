create trigger tg_stmt after insert on parent
  for statement execute procedure trig_nothing();
