create trigger t before insert or update or delete on parted
  for each row execute function parted_trigfunc();
