SELECT v as value, hashtext(v)::bit(32) as standard,
       hashtextextended(v, 0)::bit(32) as extended0,
       hashtextextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL), ('PostgreSQL'), ('eIpUEtqmY89'), ('AXKEJBTK'),
        ('muop28x03'), ('yi3nm0d73')) x(v)
WHERE  hashtext(v)::bit(32) != hashtextextended(v, 0)::bit(32)
       OR hashtext(v)::bit(32) = hashtextextended(v, 1)::bit(32);
