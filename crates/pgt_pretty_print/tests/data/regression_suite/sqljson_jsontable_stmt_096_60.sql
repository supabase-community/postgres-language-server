SELECT sub.* FROM s,
	(VALUES (23)) x(x), generate_series(13, 13) y,
	JSON_TABLE(js, '$' AS c1 PASSING x AS x, y AS y COLUMNS (
		NESTED PATH '$.a.za[2]' COLUMNS (
			NESTED PATH '$.z22[*]' as z22 COLUMNS (c int PATH '$')),
			NESTED PATH '$.a.za[1]' columns (d int[] PATH '$.z21'),
			NESTED PATH '$.a.za[0]' columns (NESTED PATH '$.z1[*]' as z1 COLUMNS (a int PATH  '$')),
			xx1 int PATH '$.c',
			NESTED PATH '$.a.za[1]'  columns (NESTED PATH '$.z21[*]' as z21 COLUMNS (b int PATH '$')),
			xx int PATH '$.c'
	)) sub;
