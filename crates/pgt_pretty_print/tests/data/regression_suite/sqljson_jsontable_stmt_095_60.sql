SELECT sub.* FROM s,
	JSON_TABLE(js, '$' PASSING 32 AS x, 13 AS y COLUMNS (
		xx int path '$.c',
		NESTED PATH '$.a.za[1]' columns (NESTED PATH '$.z21[*]' COLUMNS (z21 int path '$?(@ >= $"x")' ERROR ON ERROR))
	)) sub;
