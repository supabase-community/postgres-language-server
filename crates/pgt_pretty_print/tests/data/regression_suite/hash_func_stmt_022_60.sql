SELECT v as value, timestamp_hash(v)::bit(32) as standard,
       timestamp_hash_extended(v, 0)::bit(32) as extended0,
       timestamp_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::timestamp), ('2017-08-22 00:09:59.518762'),
        ('2015-08-20 00:11:52.51762-08'),
        ('2017-05-22 00:11:52.62-01'),
        ('2013-08-22 00:11:52.62+01'), ('2013-08-22 11:59:59+04')) x(v)
WHERE  timestamp_hash(v)::bit(32) != timestamp_hash_extended(v, 0)::bit(32)
       OR timestamp_hash(v)::bit(32) = timestamp_hash_extended(v, 1)::bit(32);
