create trigger parted_trig_before_row before insert or update or delete on parted_trig
   for each row execute procedure trigger_notice();
