create trigger parted_trig_after_stmt after insert or update or delete on parted_trig
   for each statement execute procedure trigger_notice();
