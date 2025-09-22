SELECT * FROM
  (SELECT empno,
          salary,
          dense_rank() OVER (ORDER BY salary DESC) dr
   FROM empsalary) emp
WHERE dr = 1;
