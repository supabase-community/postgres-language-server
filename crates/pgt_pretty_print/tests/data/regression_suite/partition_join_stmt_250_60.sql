insert into prtx1 select 1 + i%30, i, i
  from generate_series(1,1000) i;
