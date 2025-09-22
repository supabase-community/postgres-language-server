create trigger parent_upd_trig before update on parent
  for each row execute procedure parent_upd_func();
