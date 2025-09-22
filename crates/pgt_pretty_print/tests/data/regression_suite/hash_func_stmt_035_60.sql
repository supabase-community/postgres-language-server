SELECT v as value, hash_record(v)::bit(32) as standard
FROM   (VALUES (row('10'::varbit, 'aaa')::hash_test_t2)) x(v);
