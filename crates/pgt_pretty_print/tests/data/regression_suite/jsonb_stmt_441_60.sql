SELECT rec FROM jsonb_populate_record(
	row(NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,
		row('x',3,'2012-12-31 15:30:56')::jbpop,NULL)::jsbrec,
	'{"rec": {"a": "abc", "c": "01.02.2003", "x": 43.2}}'
) q;
