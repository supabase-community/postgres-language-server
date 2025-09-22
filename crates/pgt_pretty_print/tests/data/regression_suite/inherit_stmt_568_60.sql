select conrelid::regclass, contype, conname,
  (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
  coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass in ('inh_p3', 'inh_multiparent', 'inh_multiparent2')
 order by conrelid::regclass::text, conname;
