select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid in ('inh_parent_1'::regclass, 'inh_parent_2'::regclass, 'inh_child'::regclass)
 order by 2, conrelid::regclass::text;
