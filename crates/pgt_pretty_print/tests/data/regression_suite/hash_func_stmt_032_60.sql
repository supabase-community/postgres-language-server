SELECT v as value, hash_record(v)::bit(32) as standard,
       hash_record_extended(v, 0)::bit(32) as extended0,
       hash_record_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (row(1, 'aaa')::hash_test_t1, row(2, 'bbb'), row(-1, 'ccc'))) x(v)
WHERE  hash_record(v)::bit(32) != hash_record_extended(v, 0)::bit(32)
       OR hash_record(v)::bit(32) = hash_record_extended(v, 1)::bit(32);
