create trigger t2 before update on parted
  for each row execute function parted_trigfunc2();
