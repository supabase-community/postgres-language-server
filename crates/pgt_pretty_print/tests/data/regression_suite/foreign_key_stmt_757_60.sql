insert into fk_notpartitioned_pk (a, b)
  select 2048, x from generate_series(1,10) x;
