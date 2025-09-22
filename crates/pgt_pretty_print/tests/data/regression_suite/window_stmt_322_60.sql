SELECT
	p::text || ',' || i::text || ':' ||
		CASE WHEN f THEN COALESCE(v::text, 'NULL') ELSE '-' END as row,
	logging_agg_nonstrict(v) filter(where f) over wnd as nstrict_filt,
	logging_agg_nonstrict_initcond(v) filter(where f) over wnd as nstrict_init_filt,
	logging_agg_strict(v::text) filter(where f) over wnd as strict_filt,
	logging_agg_strict_initcond(v) filter(where f) over wnd as strict_init_filt
FROM (VALUES
	(1, 1, true,  NULL),
	(1, 2, false, 'a'),
	(1, 3, true,  'b'),
	(1, 4, false, NULL),
	(1, 5, false, NULL),
	(1, 6, false, 'c'),
	(2, 1, false, NULL),
	(2, 2, true,  'x'),
	(3, 1, true,  'z')
) AS t(p, i, f, v)
WINDOW wnd AS (PARTITION BY p ORDER BY i ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)
ORDER BY p, i;
