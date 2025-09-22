SELECT v as value, hash_aclitem(v)::bit(32) as standard,
       hash_aclitem_extended(v, 0)::bit(32) as extended0,
       hash_aclitem_extended(v, 1)::bit(32) as extended1
FROM   (SELECT DISTINCT(relacl[1]) FROM pg_class LIMIT 10) x(v)
WHERE  hash_aclitem(v)::bit(32) != hash_aclitem_extended(v, 0)::bit(32)
       OR hash_aclitem(v)::bit(32) = hash_aclitem_extended(v, 1)::bit(32);
