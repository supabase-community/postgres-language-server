SELECT
	p::text || ',' || i::text || ':' || COALESCE(v::text, 'NULL') AS row,
	logging_agg_nonstrict(v) over wnd as nstrict,
	logging_agg_nonstrict_initcond(v) over wnd as nstrict_init,
	logging_agg_strict(v::text) over wnd as strict,
	logging_agg_strict_initcond(v) over wnd as strict_init
FROM (VALUES
	(1, 1, NULL),
	(1, 2, 'a'),
	(1, 3, 'b'),
	(1, 4, NULL),
	(1, 5, NULL),
	(1, 6, 'c'),
	(2, 1, NULL),
	(2, 2, 'x'),
	(3, 1, 'z')
) AS t(p, i, v)
WINDOW wnd AS (PARTITION BY P ORDER BY i ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)
ORDER BY p, i;
