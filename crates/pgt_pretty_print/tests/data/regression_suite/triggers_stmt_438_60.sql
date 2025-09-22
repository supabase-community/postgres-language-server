create trigger trig_upd_before_child before update on parted_stmt_trig1
  for each row execute procedure trigger_notice();
