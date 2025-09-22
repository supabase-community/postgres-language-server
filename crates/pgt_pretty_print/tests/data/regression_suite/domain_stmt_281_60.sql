select count(*) from pg_constraint where contypid = 'connotnull'::regtype and contype = 'n';
