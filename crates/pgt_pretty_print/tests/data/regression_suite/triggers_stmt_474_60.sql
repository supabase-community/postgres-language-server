create trigger parted_trig_before_stmt before insert or update or delete on parted_trig
   for each statement execute procedure trigger_notice();
