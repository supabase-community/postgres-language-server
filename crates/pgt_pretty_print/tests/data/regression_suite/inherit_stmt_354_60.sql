insert into patest1
  select x, x from generate_series(0,1000) x;
