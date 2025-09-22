SELECT conname, conenforced, convalidated FROM pg_constraint
WHERE conrelid = 'fk_notpartitioned_fk'::regclass ORDER BY oid::regclass::text;
