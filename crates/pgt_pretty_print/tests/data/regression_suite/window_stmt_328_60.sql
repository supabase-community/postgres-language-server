WITH
vs AS (
	SELECT i, (random() * 100)::int4 AS v
	FROM generate_series(1, 100) AS i
),
sum_following AS (
	SELECT i, SUM(v) OVER
		(ORDER BY i DESC ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS s
	FROM vs
)
SELECT DISTINCT
	sum_following.s = sum_int_randomrestart(v) OVER fwd AS eq1,
	-sum_following.s = sum_int_randomrestart(-v) OVER fwd AS eq2,
	100*3+(vs.i-1)*3 = length(logging_agg_nonstrict(''::text) OVER fwd) AS eq3
FROM vs
JOIN sum_following ON sum_following.i = vs.i
WINDOW fwd AS (
	ORDER BY vs.i ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING
);
