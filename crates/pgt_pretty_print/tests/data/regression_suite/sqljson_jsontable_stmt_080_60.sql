SELECT *
FROM JSON_TABLE(
	jsonb '[1,2,3]',
	'$[*] ? (@ < $x)'
		PASSING 3 AS x
		COLUMNS (y text FORMAT JSON PATH '$')
	) jt;
