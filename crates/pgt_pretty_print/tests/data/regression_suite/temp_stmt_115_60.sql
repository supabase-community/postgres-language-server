create function pg_temp.twophase_func() returns void as
  $$ select '2pc_func'::text $$ language sql;
