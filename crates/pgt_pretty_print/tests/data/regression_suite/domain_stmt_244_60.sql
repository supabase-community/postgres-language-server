select conname, pg_get_constraintdef(oid) from pg_constraint
 where contypid = 'dnotnulltest'::regtype;
