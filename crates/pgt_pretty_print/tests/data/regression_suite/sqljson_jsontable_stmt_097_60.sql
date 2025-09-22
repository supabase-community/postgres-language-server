SELECT sub.* FROM s,
	(VALUES (23)) x(x), generate_series(13, 13) y,
	JSON_TABLE(js, '$' AS c1 PASSING x AS x, y AS y COLUMNS (
		xx1 int PATH '$.c',
		NESTED PATH '$.a.za[0].z1[*]' COLUMNS (NESTED PATH '$ ?(@ >= ($"x" -2))' COLUMNS (a int PATH '$')),
		NESTED PATH '$.a.za[0]' COLUMNS (NESTED PATH '$.z1[*] ? (@ >= ($"x" -2))' COLUMNS (b int PATH '$'))
	)) sub;
