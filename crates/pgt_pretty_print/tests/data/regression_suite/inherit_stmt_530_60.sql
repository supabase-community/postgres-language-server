select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_parent', 'inh_child1', 'inh_child2', 'inh_child3')
 order by 2, 1;
