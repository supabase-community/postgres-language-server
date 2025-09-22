select relname, relkind from pg_class
  where relname like 'idxpart_temp%' order by relname;
