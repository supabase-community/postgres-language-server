create trigger parted_trig_after_row after insert or update or delete on parted_trig
   for each row execute procedure trigger_notice();
