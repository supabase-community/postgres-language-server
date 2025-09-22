UPDATE pg_class
	SET reloptions = '{fillfactor=13,autovacuum_enabled=false,illegal_option=4}'
	WHERE oid = 'reloptions_test'::regclass;
