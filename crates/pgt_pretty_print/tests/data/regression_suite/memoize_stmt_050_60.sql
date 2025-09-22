SELECT explain_memoize('
SELECT * FROM prt t1 INNER JOIN prt t2 ON t1.a = t2.a;', false);
