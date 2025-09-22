select conrelid::regclass, contype, conname,
  (select attname from pg_attribute where attrelid = conrelid and attnum = conkey[1]),
  coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass in ('inh_p1', 'inh_p2', 'inh_p3', 'inh_p4',
	'inh_multiparent')
 order by conrelid::regclass::text, conname;
