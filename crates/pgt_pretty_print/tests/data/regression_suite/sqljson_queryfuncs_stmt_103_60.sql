SELECT
	JSON_QUERY(js, 'strict $[*]') AS "unspec",
	JSON_QUERY(js, 'strict $[*]' WITHOUT WRAPPER) AS "without",
	JSON_QUERY(js, 'strict $[*]' WITH CONDITIONAL WRAPPER) AS "with cond",
	JSON_QUERY(js, 'strict $[*]' WITH UNCONDITIONAL ARRAY WRAPPER) AS "with uncond",
	JSON_QUERY(js, 'strict $[*]' WITH ARRAY WRAPPER) AS "with"
FROM
	(VALUES
		(jsonb '1'),
		('[]'),
		('[null]'),
		('[12.3]'),
		('[true]'),
		('["aaa"]'),
		('[[1, 2, 3]]'),
		('[{"a": 1, "b": [2]}]'),
		('[1, "2", null, [3]]')
	) foo(js);
