create trigger t before insert or update on parted
  for each row execute function parted_trigfunc();
