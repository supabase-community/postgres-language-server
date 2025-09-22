SELECT conname, tgrelid::regclass as tgrel, regexp_replace(tgname, '[0-9]+', 'N') as tgname, tgtype
FROM pg_trigger t JOIN pg_constraint c ON (t.tgconstraint = c.oid)
WHERE tgrelid IN (SELECT relid FROM pg_partition_tree('fk_partitioned_fk'::regclass)
				  UNION ALL SELECT 'fk_notpartitioned_pk'::regclass)
ORDER BY tgrelid, tgtype;
