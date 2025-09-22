select conrelid::regclass, conname, contype, conkey,
 coninhcount, conislocal, connoinherit
 from pg_constraint where contype in ('n','p') and
 conrelid::regclass::text in ('inh_child', 'inh_parent1', 'inh_parent2')
 order by 1, 2;
