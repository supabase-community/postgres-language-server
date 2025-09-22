create trigger trig_del_after_child after delete on parted_stmt_trig1
  for each row execute procedure trigger_notice();
