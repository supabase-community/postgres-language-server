SELECT cr.relname, co.conname, co.convalidated,
       p.conname AS conparent, p.convalidated, cf.relname AS foreignrel
FROM pg_constraint co
JOIN pg_class cr ON cr.oid = co.conrelid
LEFT JOIN pg_class cf ON cf.oid = co.confrelid
LEFT JOIN pg_constraint p ON p.oid = co.conparentid
WHERE co.contype = 'f' AND
      cr.oid IN (SELECT relid FROM pg_partition_tree('parted_self_fk'))
ORDER BY cr.relname, co.conname, p.conname;
