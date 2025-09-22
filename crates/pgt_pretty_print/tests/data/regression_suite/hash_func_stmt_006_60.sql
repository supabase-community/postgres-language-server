SELECT v as value, hashchar(v)::bit(32) as standard,
       hashcharextended(v, 0)::bit(32) as extended0,
       hashcharextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::"char"), ('1'), ('x'), ('X'), ('p'), ('N')) x(v)
WHERE  hashchar(v)::bit(32) != hashcharextended(v, 0)::bit(32)
       OR hashchar(v)::bit(32) = hashcharextended(v, 1)::bit(32);
