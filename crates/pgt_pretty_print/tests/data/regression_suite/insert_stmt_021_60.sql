SELECT pg_size_pretty(pg_relation_size('large_tuple_test'::regclass, 'main'));
