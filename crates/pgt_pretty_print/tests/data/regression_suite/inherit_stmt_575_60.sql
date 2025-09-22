select conrelid::regclass, conname, conkey, coninhcount, conislocal, connoinherit
 from pg_constraint where contype = 'n' and
 conrelid::regclass::text in ('inh_nn1', 'inh_nn2', 'inh_nn3', 'inh_nn4')
 order by 2, 1;
