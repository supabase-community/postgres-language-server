create trigger trig_del_after_parent after delete on parted_stmt_trig
  for each row execute procedure trigger_notice();
