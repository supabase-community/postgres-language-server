create trigger trig_ins_after_child after insert on parted_stmt_trig1
  for each row execute procedure trigger_notice();
