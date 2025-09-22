WITH q AS (SELECT 'foo' AS x)
SELECT x, pg_typeof(x) FROM q;
