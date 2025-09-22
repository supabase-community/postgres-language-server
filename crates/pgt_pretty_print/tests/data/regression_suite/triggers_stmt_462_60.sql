create trigger zzz after insert on parted_trig for each row execute procedure trigger_notice();
