SELECT v as value, hash_numeric(v)::bit(32) as standard,
       hash_numeric_extended(v, 0)::bit(32) as extended0,
       hash_numeric_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0), (1.149484958), (17.149484958), (42.149484958),
        (149484958.550273), (2071124898672)) x(v)
WHERE  hash_numeric(v)::bit(32) != hash_numeric_extended(v, 0)::bit(32)
       OR hash_numeric(v)::bit(32) = hash_numeric_extended(v, 1)::bit(32);
