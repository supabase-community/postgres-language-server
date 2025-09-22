SELECT sub.* FROM s,
	(VALUES (23)) x(x),
	generate_series(13, 13) y,
	JSON_TABLE(js, '$' AS c1 PASSING x AS x, y AS y
	COLUMNS (
		xx1 int PATH '$.c',
		NESTED PATH '$.a.za[1]'
			COLUMNS (NESTED PATH '$.z21[*]' COLUMNS (b int PATH '$')),
		NESTED PATH '$.a.za[1] ? (@.z21[*] >= ($"x"-1))' COLUMNS
			(NESTED PATH '$.z21[*] ? (@ >= ($"y" + 3))' as z22 COLUMNS (a int PATH '$ ? (@ >= ($"y" + 12))')),
		NESTED PATH '$.a.za[1]' COLUMNS
			(NESTED PATH '$.z21[*] ? (@ >= ($"y" +121))' as z21 COLUMNS (c int PATH '$ ? (@ > ($"x" +111))'))
	)) sub;
