insert into parted_conflict values(0, 'cero', 1)
  on conflict (a,b) do update set c = parted_conflict.c + 1;
