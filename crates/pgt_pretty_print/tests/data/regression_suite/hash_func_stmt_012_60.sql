SELECT v as value, hashinet(v)::bit(32) as standard,
       hashinetextended(v, 0)::bit(32) as extended0,
       hashinetextended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::inet), ('192.168.100.128/25'), ('192.168.100.0/8'),
        ('172.168.10.126/16'), ('172.18.103.126/24'), ('192.188.13.16/32')) x(v)
WHERE  hashinet(v)::bit(32) != hashinetextended(v, 0)::bit(32)
       OR hashinet(v)::bit(32) = hashinetextended(v, 1)::bit(32);
