SELECT v as value, hashint2(v)::bit(32) as standard,
       hashint2extended(v, 0)::bit(32) as extended0,
       hashint2extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0::int2), (1::int2), (17::int2), (42::int2)) x(v)
WHERE  hashint2(v)::bit(32) != hashint2extended(v, 0)::bit(32)
       OR hashint2(v)::bit(32) = hashint2extended(v, 1)::bit(32);
