create domain insert_nnarray as int[]
  check (value[1] is not null and value[2] is not null);
