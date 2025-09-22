SELECT conname, description
FROM  pg_description, pg_constraint c
WHERE classoid = 'pg_constraint'::regclass
AND   objoid = c.oid AND c.conrelid = 'noinh_con_copy1'::regclass
ORDER BY conname COLLATE "C";
