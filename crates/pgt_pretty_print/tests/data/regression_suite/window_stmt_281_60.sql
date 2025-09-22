SELECT * FROM
  (SELECT empno,
          salary,
          count(*) OVER (ORDER BY salary DESC) c,
          dense_rank() OVER (ORDER BY salary DESC) dr
   FROM empsalary) emp
WHERE dr = 1;
