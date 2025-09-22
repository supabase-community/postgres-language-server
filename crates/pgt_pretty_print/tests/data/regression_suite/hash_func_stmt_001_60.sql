SELECT v as value, hashint4(v)::bit(32) as standard,
       hashint4extended(v, 0)::bit(32) as extended0,
       hashint4extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0), (1), (17), (42), (550273), (207112489)) x(v)
WHERE  hashint4(v)::bit(32) != hashint4extended(v, 0)::bit(32)
       OR hashint4(v)::bit(32) = hashint4extended(v, 1)::bit(32);
