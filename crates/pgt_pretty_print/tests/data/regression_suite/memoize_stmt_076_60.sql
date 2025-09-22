SELECT explain_memoize('
SELECT COUNT(*) FROM tab_anti t1 LEFT JOIN
LATERAL (SELECT DISTINCT ON (a) a, b, t1.a AS x FROM tab_anti t2) t2
ON t1.a+1 = t2.a
WHERE t2.a IS NULL;', false);
