CREATE VIEW jsonb_table_view4 AS
SELECT * FROM
	JSON_TABLE(
		jsonb 'null', 'lax $[*]' PASSING 1 + 2 AS a, json '"foo"' AS "b c"
		COLUMNS (
            jsb jsonb   FORMAT JSON PATH '$',
            jsbq jsonb FORMAT JSON PATH '$' OMIT QUOTES,
            aaa int, -- implicit path '$."aaa"',
            aaa1 int PATH '$.aaa'));
