insert into permtest_parent
  select 1, 'a', left(fipshash(i::text), 5) from generate_series(0, 100) i;
