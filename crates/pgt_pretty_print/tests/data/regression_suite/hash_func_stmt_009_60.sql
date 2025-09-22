SELECT v as value, hashoidvector(v)::bit(32) as standard,
       hashoidvectorextended(v, 0)::bit(32) as extended0,
       hashoidvectorextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::oidvector), ('0 1 2 3 4'), ('17 18 19 20'),
        ('42 43 42 45'), ('550273 550273 570274'),
        ('207112489 207112499 21512 2155 372325 1363252')) x(v)
WHERE  hashoidvector(v)::bit(32) != hashoidvectorextended(v, 0)::bit(32)
       OR hashoidvector(v)::bit(32) = hashoidvectorextended(v, 1)::bit(32);
