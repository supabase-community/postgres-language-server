SELECT v as value, hashenum(v)::bit(32) as standard,
       hashenumextended(v, 0)::bit(32) as extended0,
       hashenumextended(v, 1)::bit(32) as extended1
FROM   (VALUES ('sad'::mood), ('ok'), ('happy')) x(v)
WHERE  hashenum(v)::bit(32) != hashenumextended(v, 0)::bit(32)
       OR hashenum(v)::bit(32) = hashenumextended(v, 1)::bit(32);
