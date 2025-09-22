insert into parted_conflict_1 values (40, 'cuarenta')
  on conflict (a) do update set b = excluded.b;
