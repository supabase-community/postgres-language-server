create index concurrently test_pg_index_toast_index
  on test_pg_index_toast_table (test_pg_index_toast_func(a, 'b'));
