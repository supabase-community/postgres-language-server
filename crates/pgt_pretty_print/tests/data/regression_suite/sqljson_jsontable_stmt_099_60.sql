SELECT sub.* FROM s,
	(values(23)) x(x),
	generate_series(13, 13) y,
	JSON_TABLE(js, '$' AS c1 PASSING x AS x, y AS y
	COLUMNS (
		xx1 int PATH '$.c',
		NESTED PATH '$.a.za[2]' COLUMNS (NESTED PATH '$.z22[*]' as z22 COLUMNS (c int PATH '$')),
		NESTED PATH '$.a.za[1]' COLUMNS (d json PATH '$ ? (@.z21[*] == ($"x" -1))'),
		NESTED PATH '$.a.za[0]' COLUMNS (NESTED PATH '$.z1[*] ? (@ >= ($"x" -2))' as z1 COLUMNS (a int PATH '$')),
		NESTED PATH '$.a.za[1]' COLUMNS
			(NESTED PATH '$.z21[*] ? (@ >= ($"y" +121))' as z21 COLUMNS (b int PATH '$ ? (@ > ($"x" +111))' DEFAULT 0 ON EMPTY))
	)) sub;
