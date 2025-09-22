SELECT v as value, timetz_hash(v)::bit(32) as standard,
       timetz_hash_extended(v, 0)::bit(32) as extended0,
       timetz_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::timetz), ('00:11:52.518762-07'), ('00:11:52.51762-08'),
        ('00:11:52.62-01'), ('00:11:52.62+01'), ('11:59:59+04')) x(v)
WHERE  timetz_hash(v)::bit(32) != timetz_hash_extended(v, 0)::bit(32)
       OR timetz_hash(v)::bit(32) = timetz_hash_extended(v, 1)::bit(32);
