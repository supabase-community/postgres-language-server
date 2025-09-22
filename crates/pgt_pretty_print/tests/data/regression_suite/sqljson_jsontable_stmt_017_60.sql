SELECT *
FROM json_table_test vals
	LEFT OUTER JOIN
	JSON_TABLE(
		vals.js::jsonb, 'lax $[*]'
		COLUMNS (
			id FOR ORDINALITY,
			exists1 bool EXISTS PATH '$.aaa',
			exists2 int EXISTS PATH '$.aaa',
			exists3 int EXISTS PATH 'strict $.aaa' UNKNOWN ON ERROR,
			exists4 text EXISTS PATH 'strict $.aaa' FALSE ON ERROR
		)
	) jt
	ON true;
