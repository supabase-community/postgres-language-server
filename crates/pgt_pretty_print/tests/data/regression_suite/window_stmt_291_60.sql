SELECT 1 FROM
  (SELECT ntile(e2.salary) OVER (PARTITION BY e1.depname) AS c
   FROM empsalary e1 LEFT JOIN empsalary e2 ON TRUE
   WHERE e1.empno = e2.empno) s
WHERE s.c = 1;
