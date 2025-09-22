declare c1 scroll cursor for
 select * from generate_series(1,4) i
  where i <> all (values (2),(3));
