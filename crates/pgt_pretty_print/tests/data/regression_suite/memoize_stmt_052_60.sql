SELECT explain_memoize('
SELECT * FROM prt_p1 t1 INNER JOIN
(SELECT * FROM prt_p1 UNION ALL SELECT * FROM prt_p2) t2
ON t1.a = t2.a;', false);
