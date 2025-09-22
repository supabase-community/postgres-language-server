SELECT pg_relation_size('test_io_local') / current_setting('block_size')::int8 > 100;
