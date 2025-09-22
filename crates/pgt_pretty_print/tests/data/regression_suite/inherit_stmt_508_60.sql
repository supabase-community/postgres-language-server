select conrelid::regclass, conname, contype, conkey,
 (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
 coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text like 'inh\_nn\_%'
 order by 2, 1;
