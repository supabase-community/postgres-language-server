SELECT conname, conenforced, convalidated FROM pg_constraint
WHERE conrelid = 'fk_partitioned_pk'::regclass AND contype = 'f'
ORDER BY oid::regclass::text;
