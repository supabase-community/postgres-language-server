SELECT
	JSON_QUERY(js, '$') AS "unspec",
	JSON_QUERY(js, '$' WITHOUT WRAPPER) AS "without",
	JSON_QUERY(js, '$' WITH CONDITIONAL WRAPPER) AS "with cond",
	JSON_QUERY(js, '$' WITH UNCONDITIONAL ARRAY WRAPPER) AS "with uncond",
	JSON_QUERY(js, '$' WITH ARRAY WRAPPER) AS "with"
FROM
	(VALUES
		(jsonb 'null'),
		('12.3'),
		('true'),
		('"aaa"'),
		('[1, null, "2"]'),
		('{"a": 1, "b": [2]}')
	) foo(js);
