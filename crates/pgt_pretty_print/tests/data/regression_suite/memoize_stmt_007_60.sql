SELECT explain_memoize('
SELECT COUNT(*),AVG(t2.t1two) FROM tenk1 t1 LEFT JOIN
LATERAL (
    SELECT t1.two as t1two, * FROM tenk1 t2 WHERE t2.unique1 < 4 OFFSET 0
) t2
ON t1.two = t2.two
WHERE t1.unique1 < 10;', false);
