SELECT
	logging_agg_strict(v::text) OVER wnd
FROM (VALUES
	(1, 'a'),
	(2, 'b'),
	(3, 'c')
) AS t(i, v)
WINDOW wnd AS (ORDER BY i ROWS BETWEEN CURRENT ROW AND CURRENT ROW)
ORDER BY i;
