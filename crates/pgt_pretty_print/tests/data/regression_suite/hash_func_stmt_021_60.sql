SELECT v as value, interval_hash(v)::bit(32) as standard,
       interval_hash_extended(v, 0)::bit(32) as extended0,
       interval_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::interval),
        ('5 month 7 day 46 minutes'), ('1 year 7 day 46 minutes'),
        ('1 year 7 month 20 day 46 minutes'), ('5 month'),
        ('17 year 11 month 7 day 9 hours 46 minutes 5 seconds')) x(v)
WHERE  interval_hash(v)::bit(32) != interval_hash_extended(v, 0)::bit(32)
       OR interval_hash(v)::bit(32) = interval_hash_extended(v, 1)::bit(32);
