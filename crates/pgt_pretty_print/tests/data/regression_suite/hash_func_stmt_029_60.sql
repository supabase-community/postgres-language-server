SELECT v as value, hash_range(v)::bit(32) as standard,
       hash_range_extended(v, 0)::bit(32) as extended0,
       hash_range_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (int4range(10, 20)), (int4range(23, 43)),
        (int4range(5675, 550273)),
        (int4range(550274, 1550274)), (int4range(1550275, 208112489))) x(v)
WHERE  hash_range(v)::bit(32) != hash_range_extended(v, 0)::bit(32)
       OR hash_range(v)::bit(32) = hash_range_extended(v, 1)::bit(32);
