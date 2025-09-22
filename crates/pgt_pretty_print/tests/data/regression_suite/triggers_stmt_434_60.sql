create trigger trig_upd_after_parent after update on parted_stmt_trig
  for each row execute procedure trigger_notice();
