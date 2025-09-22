SELECT * FROM
  (SELECT empno,
          salary,
          count(*) OVER (ORDER BY salary) c
   FROM empsalary) emp
WHERE 3 <= c;
