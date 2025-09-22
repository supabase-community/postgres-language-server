create trigger aaa after insert on parted_trig
   for each row execute procedure trigger_notice('quirky', 1);
