select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid in ('inh_parent'::regclass, 'inh_child1'::regclass, 'inh_child2'::regclass, 'inh_child3'::regclass)
 order by 2, conrelid::regclass::text;
