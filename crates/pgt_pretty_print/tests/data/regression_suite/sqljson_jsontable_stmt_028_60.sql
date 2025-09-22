CREATE VIEW jsonb_table_view6 AS
SELECT * FROM
	JSON_TABLE(
		jsonb 'null', 'lax $[*]' PASSING 1 + 2 AS a, json '"foo"' AS "b c"
		COLUMNS (
			js2 json PATH '$',
			jsb2w jsonb PATH '$' WITH WRAPPER,
			jsb2q jsonb PATH '$' OMIT QUOTES,
			ia int[] PATH '$',
			ta text[] PATH '$',
			jba jsonb[] PATH '$'));
