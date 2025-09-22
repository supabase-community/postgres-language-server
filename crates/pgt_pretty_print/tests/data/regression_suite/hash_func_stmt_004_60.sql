SELECT v as value, hashfloat8(v)::bit(32) as standard,
       hashfloat8extended(v, 0)::bit(32) as extended0,
       hashfloat8extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0), (1), (17), (42), (550273), (207112489)) x(v)
WHERE  hashfloat8(v)::bit(32) != hashfloat8extended(v, 0)::bit(32)
       OR hashfloat8(v)::bit(32) = hashfloat8extended(v, 1)::bit(32);
