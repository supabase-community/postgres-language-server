SELECT v as value, hashname(v)::bit(32) as standard,
       hashnameextended(v, 0)::bit(32) as extended0,
       hashnameextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL), ('PostgreSQL'), ('eIpUEtqmY89'), ('AXKEJBTK'),
        ('muop28x03'), ('yi3nm0d73')) x(v)
WHERE  hashname(v)::bit(32) != hashnameextended(v, 0)::bit(32)
       OR hashname(v)::bit(32) = hashnameextended(v, 1)::bit(32);
