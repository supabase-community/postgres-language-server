SELECT v as value, hashbpchar(v)::bit(32) as standard,
       hashbpcharextended(v, 0)::bit(32) as extended0,
       hashbpcharextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL), ('PostgreSQL'), ('eIpUEtqmY89'), ('AXKEJBTK'),
        ('muop28x03'), ('yi3nm0d73')) x(v)
WHERE  hashbpchar(v)::bit(32) != hashbpcharextended(v, 0)::bit(32)
       OR hashbpchar(v)::bit(32) = hashbpcharextended(v, 1)::bit(32);
