SELECT v as value, hash_array(v)::bit(32) as standard,
       hash_array_extended(v, 0)::bit(32) as extended0,
       hash_array_extended(v, 1)::bit(32) as extended1
FROM   (VALUES ('{0}'::int4[]), ('{0,1,2,3,4}'), ('{17,18,19,20}'),
        ('{42,34,65,98}'), ('{550273,590027, 870273}'),
        ('{207112489, 807112489}')) x(v)
WHERE  hash_array(v)::bit(32) != hash_array_extended(v, 0)::bit(32)
       OR hash_array(v)::bit(32) = hash_array_extended(v, 1)::bit(32);
