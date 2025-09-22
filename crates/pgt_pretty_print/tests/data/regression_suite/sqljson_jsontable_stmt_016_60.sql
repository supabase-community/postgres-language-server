SELECT *
FROM json_table_test vals
	LEFT OUTER JOIN
	JSON_TABLE(
		vals.js::jsonb, 'lax $[*]'
		COLUMNS (
			id FOR ORDINALITY,
			jst text    FORMAT JSON  PATH '$',
			jsc char(4) FORMAT JSON  PATH '$',
			jsv varchar(4) FORMAT JSON  PATH '$',
			jsb jsonb FORMAT JSON PATH '$',
			jsbq jsonb FORMAT JSON PATH '$' OMIT QUOTES
		)
	) jt
	ON true;
