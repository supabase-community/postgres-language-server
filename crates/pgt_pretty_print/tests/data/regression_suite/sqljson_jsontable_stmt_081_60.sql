SELECT *
FROM JSON_TABLE(
	jsonb '[1,2,3]',
	'$[*] ? (@ < $x)'
		PASSING 10 AS x, 3 AS y
		COLUMNS (a text FORMAT JSON PATH '$ ? (@ < $y)')
	) jt;
