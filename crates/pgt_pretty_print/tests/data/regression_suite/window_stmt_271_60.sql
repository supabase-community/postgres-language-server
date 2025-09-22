SELECT * FROM
  (SELECT empno,
          salary,
          rank() OVER (ORDER BY salary DESC) r
   FROM empsalary) emp
WHERE r <= 3;
