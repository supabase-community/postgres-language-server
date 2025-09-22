SELECT v as value, time_hash(v)::bit(32) as standard,
       time_hash_extended(v, 0)::bit(32) as extended0,
       time_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::time), ('11:09:59'), ('1:09:59'), ('11:59:59'),
        ('7:9:59'), ('5:15:59')) x(v)
WHERE  time_hash(v)::bit(32) != time_hash_extended(v, 0)::bit(32)
       OR time_hash(v)::bit(32) = time_hash_extended(v, 1)::bit(32);
