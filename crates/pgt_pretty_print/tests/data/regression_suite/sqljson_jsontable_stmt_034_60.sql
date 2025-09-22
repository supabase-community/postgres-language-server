SELECT * FROM
	JSON_TABLE(
		jsonb 'null', 'lax $[*]' PASSING 1 + 2 AS a, json '"foo"' AS "b c"
		COLUMNS (
			id FOR ORDINALITY,
			"int" int PATH '$',
			"text" text PATH '$'
	)) json_table_func;
