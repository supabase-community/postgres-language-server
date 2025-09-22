SELECT pg_describe_object('pg_constraint'::regclass, oid, 0), confrelid::regclass,
       CASE WHEN conparentid <> 0 THEN pg_describe_object('pg_constraint'::regclass, conparentid, 0) ELSE 'TOP' END
FROM pg_catalog.pg_constraint
WHERE conrelid IN (SELECT relid FROM pg_partition_tree('fk'))
ORDER BY conrelid::regclass::text, conname;
