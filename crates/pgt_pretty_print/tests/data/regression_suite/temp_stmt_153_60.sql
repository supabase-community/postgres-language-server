SELECT pg_relation_size('test_temp') / current_setting('block_size')::int8 > 200;
