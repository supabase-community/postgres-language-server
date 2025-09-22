SELECT v as value, hashmacaddr8(v)::bit(32) as standard,
       hashmacaddr8extended(v, 0)::bit(32) as extended0,
       hashmacaddr8extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::macaddr8), ('08:00:2b:01:02:04:36:49'),
        ('08:00:2b:01:02:04:f0:e8'), ('e2:7f:51:3e:70:49:16:29'),
        ('d6:a9:4a:78:1c:d5:47:32'), ('ea:29:b1:5e:1f:a5')) x(v)
WHERE  hashmacaddr8(v)::bit(32) != hashmacaddr8extended(v, 0)::bit(32)
       OR hashmacaddr8(v)::bit(32) = hashmacaddr8extended(v, 1)::bit(32);
