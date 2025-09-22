SELECT
	JSON_ARRAYAGG(bar) as no_options,
	JSON_ARRAYAGG(bar RETURNING jsonb) as returning_jsonb,
	JSON_ARRAYAGG(bar ABSENT ON NULL) as absent_on_null,
	JSON_ARRAYAGG(bar ABSENT ON NULL RETURNING jsonb) as absentonnull_returning_jsonb,
	JSON_ARRAYAGG(bar NULL ON NULL) as null_on_null,
	JSON_ARRAYAGG(bar NULL ON NULL RETURNING jsonb) as nullonnull_returning_jsonb,
	JSON_ARRAYAGG(foo) as row_no_options,
	JSON_ARRAYAGG(foo RETURNING jsonb) as row_returning_jsonb,
	JSON_ARRAYAGG(foo ORDER BY bar) FILTER (WHERE bar > 2) as row_filtered_agg,
	JSON_ARRAYAGG(foo ORDER BY bar RETURNING jsonb) FILTER (WHERE bar > 2) as row_filtered_agg_returning_jsonb
FROM
	(VALUES (NULL), (3), (1), (NULL), (NULL), (5), (2), (4), (NULL)) foo(bar);
