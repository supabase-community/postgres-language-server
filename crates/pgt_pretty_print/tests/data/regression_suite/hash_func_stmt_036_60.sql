SELECT v as value, hash_record_extended(v, 0)::bit(32) as extended0
FROM   (VALUES (row('11'::varbit, 'aaa')::hash_test_t2)) x(v);
