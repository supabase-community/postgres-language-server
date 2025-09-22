select relname from pg_class where relname ~ '^temp_parted_oncommit_test'
  order by relname;
