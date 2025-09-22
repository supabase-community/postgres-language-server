SELECT v as value, hashfloat4(v)::bit(32) as standard,
       hashfloat4extended(v, 0)::bit(32) as extended0,
       hashfloat4extended(v, 1)::bit(32) as extended1
FROM   (VALUES (0), (1), (17), (42), (550273), (207112489)) x(v)
WHERE  hashfloat4(v)::bit(32) != hashfloat4extended(v, 0)::bit(32)
       OR hashfloat4(v)::bit(32) = hashfloat4extended(v, 1)::bit(32);
