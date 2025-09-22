select conrelid::regclass, conname, contype, coninhcount, conislocal
 from pg_constraint
 where conrelid::regclass::text in ('inh_parent', 'inh_child', 'inh_child2', 'inh_child3')
 order by 2, 1;
