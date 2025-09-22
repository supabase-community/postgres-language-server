SELECT v as value, hashmacaddr(v)::bit(32) as standard,
       hashmacaddrextended(v, 0)::bit(32) as extended0,
       hashmacaddrextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::macaddr), ('08:00:2b:01:02:04'), ('08:00:2b:01:02:04'),
        ('e2:7f:51:3e:70:49'), ('d6:a9:4a:78:1c:d5'),
        ('ea:29:b1:5e:1f:a5')) x(v)
WHERE  hashmacaddr(v)::bit(32) != hashmacaddrextended(v, 0)::bit(32)
       OR hashmacaddr(v)::bit(32) = hashmacaddrextended(v, 1)::bit(32);
