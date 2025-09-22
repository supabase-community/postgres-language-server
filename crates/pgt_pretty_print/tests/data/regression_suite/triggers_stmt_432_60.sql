create trigger trig_del_after after delete on parted_stmt_trig
  for each statement execute procedure trigger_notice();
