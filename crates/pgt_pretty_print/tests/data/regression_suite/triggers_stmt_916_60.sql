create trigger aft_row after insert or update on trigger_parted
  for each row execute function trigger_parted_trigfunc();
