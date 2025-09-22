create trigger parted_trig after insert on parted_irreg
  for each row execute procedure trigger_notice_ab();
