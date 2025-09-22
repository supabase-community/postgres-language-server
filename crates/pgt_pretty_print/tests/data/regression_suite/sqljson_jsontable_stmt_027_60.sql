CREATE VIEW jsonb_table_view5 AS
SELECT * FROM
	JSON_TABLE(
		jsonb 'null', 'lax $[*]' PASSING 1 + 2 AS a, json '"foo"' AS "b c"
		COLUMNS (
			exists1 bool EXISTS PATH '$.aaa',
			exists2 int EXISTS PATH '$.aaa' TRUE ON ERROR,
			exists3 text EXISTS PATH 'strict $.aaa' UNKNOWN ON ERROR));
