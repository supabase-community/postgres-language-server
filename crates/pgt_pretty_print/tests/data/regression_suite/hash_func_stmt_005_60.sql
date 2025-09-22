SELECT v as value, hashoid(v)::bit(32) as standard,
       hashoidextended(v, 0)::bit(32) as extended0,
       hashoidextended(v, 1)::bit(32) as extended1
FROM   (VALUES (0), (1), (17), (42), (550273), (207112489)) x(v)
WHERE  hashoid(v)::bit(32) != hashoidextended(v, 0)::bit(32)
       OR hashoid(v)::bit(32) = hashoidextended(v, 1)::bit(32);
