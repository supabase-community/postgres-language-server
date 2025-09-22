create trigger trig_ins_before before insert on parted_stmt_trig
  for each statement execute procedure trigger_notice();
