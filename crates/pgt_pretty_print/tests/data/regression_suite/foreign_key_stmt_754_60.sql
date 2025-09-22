insert into other_partitioned_fk
  select 2048, x from generate_series(1,10) x;
